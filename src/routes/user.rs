use axum::extract::Query;
use axum::response::{Html, Redirect};
use axum::routing::get;
use axum::Extension;
use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use entity::user;
use migration::sea_orm::{Database, DatabaseConnection, EntityTrait, QueryFilter, Set};
use models::user_models::{CreateUserModel, GetAllUsersModel, UserModelPub};

use crate::models;
use dotenv::dotenv;
use migration::sea_orm::ColumnTrait;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use std::any::Any;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub fn user_routes(db: Arc<DatabaseConnection>) -> Router {
    //tenant - dev-osgutng6i3uptora
    //domain - dev-osgutng6i3uptora.auth0.com

    Router::new()
        .route("/users", get(get_all_users))
        .route("/user/insert", post(create_user))
        .route("/privacy", get( || async {"Privacy Policy"}))
        .route("/tos", get( || async {"TOS"}))
        .layer(Extension(db))
        .merge(auth_user_routes())
}

pub fn auth_user_routes() -> Router {
    let oauth_client = create_oauth_client();
    Router::new()
        .route("/auth", get(auth))
        .route("/redirect", get(redirect_auth))
        .layer(Extension(oauth_client))
}

async fn get_all_users(Extension(db): Extension<Arc<DatabaseConnection>>) -> impl IntoResponse {
    let users = entity::user::Entity::find().all(db.as_ref()).await;

    match users {
        Ok(res) => (
            StatusCode::OK,
            Json(GetAllUsersModel {
                users: res
                    .iter()
                    .map(|u| UserModelPub {
                        name: (*u.name).to_string(),
                        email: (*u.email).to_string(),
                    })
                    .collect(),
            }),
        ),
        Err(_) => (StatusCode::NOT_FOUND, Json::default()),
    }
}

async fn create_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    user_data: Json<CreateUserModel>,
) -> impl IntoResponse {
    let user_model = user::ActiveModel {
        name: Set(user_data.name.to_owned()),
        email: Set(user_data.email.to_owned()),
        uuid: Set(Uuid::new_v4()),
        ..Default::default()
    };

    user::Entity::insert(user_model.clone())
        .exec(db.as_ref())
        .await
        .unwrap();

    (StatusCode::CREATED, format!("{:?}", user_model.uuid))
}

//TODO: Upload Image

//TODO: Add OAuth2.0, store sessions in DB, add feature of Google log in

async fn auth(Extension(oauth_client): Extension<Arc<Mutex<BasicClient>>>) -> impl IntoResponse {
    let (auth_url, csrf_token) = oauth_client
        .lock()
        .await
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "openid".to_string()
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.email".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.profile".to_string(),
        ))
        .url();

    println!("CSRF token: {}", csrf_token.secret());
    println!("Authorization URL: {}", auth_url);
    Redirect::temporary(&auth_url.to_string())
}

async fn redirect_auth(
    Query(params): Query<HashMap<String, String>>,
    Extension(oauth_client): Extension<Arc<Mutex<BasicClient>>>,
) -> impl IntoResponse {
    println!("Received query parameters: {:?}", params);

    if let Some(code) = params.get("code") {
        let token_result = oauth_client
            .lock()
            .await
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await;

        match token_result {
            Ok(token) => {
                let access_token = token.access_token().secret();
                let refresh_token = token.refresh_token().unwrap().secret();
                
                

                // Use Bearer token in the Authorization header
                let url = "https://www.googleapis.com/oauth2/v2/userinfo?oauth_token=".to_owned() + access_token;

                println!("Access Token: {:?}", access_token);

                let body_text = reqwest::get(url)
                    .await
                    .map_err(|_| "OAuth: reqwest failed to query userinfo")
                    .expect("Something bad happened")
                    .text()
                    .await
                    .unwrap();

                println!("Userinfo response: {}", body_text);

                let body: serde_json::Value = serde_json::from_str(&body_text)
                    .map_err(|_| "OAuth: Serde failed to parse userinfo")
                    .expect("Failed to parse userinfo response");

                let email = body["email"]
                    .as_str()
                    .ok_or("Oauth: Failed to parse email form address")
                    .map(|s| s.to_owned());

                if let Ok(email) = email {
                    let verified_email = body["verified_email"]
                        .as_bool()
                        .ok_or("Oauth: Serde failed to parse verified email")
                        .unwrap();

                    if !verified_email {
                        return Html("Email is not verified.".to_string());
                    }

                    dotenv().ok();
                    let db = env::var("DATABASE_URL").expect("Failed to load db");
                    let db_conn = Database::connect(&db).await.unwrap();

                    let query = entity::user::Entity::find()
                        .filter(entity::user::Column::Email.eq(email.clone()))
                        .one(&db_conn)
                        .await;

                    match query {
                        Ok(Some(user)) => {
                            Html(format!("User found: {:?}", user))
                            
                        
                        },
                        Ok(None) => Html("User not found".to_string()),
                        Err(err) => Html(format!("Database query failed: {:?}", err)),
                    }
                } else {
                    Html("Failed to parse email from userinfo response".to_string())
                }
            }
            Err(err) => {
                eprintln!("Failed to exchange code: {:?}", err);
                Html("Failed to exchange code".to_string())
            }
        }
    } else {
        println!("Missing code parameter");
        Html("Missing code".to_string())
    }
}

fn create_oauth_client() -> Arc<Mutex<BasicClient>> {
    let client_id = ClientId::new(env::var("GOOGLE_OAUTH_CLIENT_ID").expect("Missing CLIENT_ID"));
    let client_secret =
        ClientSecret::new(env::var("GOOGLE_OAUTH_CLIENT_SECRET").expect("Missing CLIENT_SECRET"));
    let auth_url = AuthUrl::new(env::var("OAUTH_AUTH_URL").expect("Missing AUTH_URL"))
        .expect("Invalid AUTH_URL");
    let token_url = TokenUrl::new(env::var("OAUTH_TOKEN_URL").expect("Missing TOKEN_URL"))
        .expect("Invalid TOKEN_URL");
    let redirect_url =
        RedirectUrl::new(env::var("OAUTH_REDIRECT_URL").expect("Missing REDIRECT_URL"))
            .expect("Invalid REDIRECT_URL");

    let oauth_client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url);

    Arc::new(Mutex::new(oauth_client))
}

//TODO: Add log out
