use axum::extract::Query;
use axum::response::{Html, Redirect};
use axum::routing::get;
use axum::Extension;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use entity::user;
use models::user_models::{GetAllUsersModel, UserModelPub, CreateUserModel};
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, AuthUrl, TokenUrl, RedirectUrl,
    CsrfToken, Scope, basic::BasicClient, TokenResponse
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use uuid::Uuid;
use crate::models;


pub fn user_routes(db: Arc<DatabaseConnection>) -> Router {


    //tenant - dev-osgutng6i3uptora
    //domain - dev-osgutng6i3uptora.auth0.com

    Router::new()
        .route("/users", get(get_all_users))
        .route("/user/insert", post(create_user))
        .layer(Extension(db))
        .merge(auth_user_routes())
} 

pub fn auth_user_routes() -> Router{
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

async fn create_user(Extension(db): Extension<Arc<DatabaseConnection>>, user_data: Json<CreateUserModel>) -> impl IntoResponse {


    let user_model = user::ActiveModel {
        name: ActiveValue::Set(user_data.name.to_owned()),
        email: ActiveValue::Set(user_data.email.to_owned()),
        password: ActiveValue::Set(user_data.password.to_owned()),
        uuid: ActiveValue::Set(Uuid::new_v4()),
        ..Default::default()
    };

    user_model.clone().insert(db.as_ref()).await.unwrap();

    (StatusCode::ACCEPTED, format!("{:?}", user_model.uuid))
}

//TODO: Upload Image 



//TODO: Add OAuth2.0, store sessions in DB, add feature of Google log in 

async fn auth(Extension(oauth_client): Extension<Arc<Mutex<BasicClient>>>) -> impl IntoResponse {
    let (auth_url, csrf_token) = oauth_client.lock().await
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
        .url();

    println!("CSRF token: {}", csrf_token.secret());
    println!("Authorization URL: {}", auth_url);
    Redirect::temporary(&auth_url.to_string())
}

async fn redirect_auth(
    Query(params): Query<HashMap<String, String>>,
    Extension(oauth_client): Extension<Arc<Mutex<BasicClient>>>,
) -> impl IntoResponse {
    // Log the received query parameters
    println!("Received query parameters: {:?}", params);

    if let Some(code) = params.get("code") {
        let token_result = oauth_client.lock().await
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await;

        match token_result {
            Ok(token) => {
                let access_token = token.access_token();
                println!("Access Token: {:?}", access_token.secret());
                Html(format!("Access Token: {:?}", access_token.secret()))
            }
            Err(err) => {
                eprintln!("Failed to exchange code: {:?}", err);
                Html("Failed to exchange code".to_string())
            }
        }
    } else {
        // Log if the code is missing
        println!("Missing code parameter");
        Html("Missing code".to_string())
    }
}


fn create_oauth_client() -> Arc<Mutex<BasicClient>> {


    let client_id = ClientId::new(env::var("CLIENT_ID").expect("Missing CLIENT_ID"));
    let client_secret = ClientSecret::new(env::var("CLIENT_SECRET").expect("Missing CLIENT_SECRET"));
    let auth_url = AuthUrl::new(env::var("OAUTH_AUTH_URL").expect("Missing AUTH_URL")).expect("Invalid AUTH_URL");
    let token_url = TokenUrl::new(env::var("OAUTH_TOKEN_URL").expect("Missing TOKEN_URL")).expect("Invalid TOKEN_URL");
    let redirect_url = RedirectUrl::new(env::var("OAUTH_REDIRECT_URL").expect("Missing REDIRECT_URL")).expect("Invalid REDIRECT_URL");

    let oauth_client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url);

    Arc::new(Mutex::new(oauth_client))
}

//TODO: Add log out
