use crate::redis_manager::session_setting::{get_session_id, set_session_id};
use axum::body::Body;
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::response::{Html, Redirect};
use axum::routing::get;
use axum::Extension;
use axum::{response::IntoResponse, Router};

use axum_extra::headers;
use axum_extra::TypedHeader;

use chrono::{Duration, Utc};
use migration::sea_orm::ColumnTrait;
use migration::sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Set};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use reqwest::header;

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_cookies::cookie::SameSite;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;


use super::middlewares::user_expired;

pub fn auth_user_routes(db: Arc<DatabaseConnection>) -> Router {
    let oauth_client = create_oauth_client();

    Router::new()
        .route("/auth", get(auth))
        .route("/redirect", get(redirect_auth))
        .route("/dashboard", get(dashboard))
        .route("/login", get(login))
        .route("/logout", get(logout))
        .layer(axum::middleware::from_fn(user_expired))
        .layer(Extension(oauth_client))
        .layer(Extension(db))

}

async fn login() -> impl IntoResponse {
    Html(
       r#"
           <form action="http://localhost:3010/auth_sign_on">
               <input type="submit" value="Sign Up With Google" />
           </form>

           <form action="http://localhost:3010/auth">
               <input type="submit" value="Login With Google" />
           </form>
       "#,
    )
}

async fn dashboard(
    cookie_header: Option<TypedHeader<headers::Cookie>>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> impl IntoResponse {
    let mut name = "".to_string();
    let mut email = "".to_string();

    //Take the existing session from the cookie
    if let Some(TypedHeader(ref cookie)) = cookie_header {
        //Find the session_id in session table
        if let Some(session_id) = cookie.get("session_id") {
            // Attempt to convert the session_id to a UUID

            if get_session_id(session_id.to_string()).await.unwrap_or(false) {
                println!("Session exists");
            } else {
                println!("Session does not exist");
                return Redirect::temporary("/login").into_response();
            }

            let session_uuid = match Uuid::parse_str(session_id) {
                Ok(uuid) => uuid,
                Err(err) => {
                    println!("Failed to parse session_id as UUID: {}", err);
                    return Html("Invalid session ID format.".to_string()).into_response();
                }
            };

            // Query the database for the session
            //Get the user_id from the session table
            let session_query = entity::session::Entity::find()
                .filter(entity::session::Column::SessionId.eq(session_uuid))
                .one(db.as_ref())
                .await;

            if let Ok(Some(session)) = session_query {
                println!("Session found in database: {:?}", session);

                //Select the user
                //Compare the existing user_id from the session to id in user table
                let user_query = entity::user::Entity::find()
                    .filter(entity::user::Column::Uuid.eq(session.user_id))
                    .one(db.as_ref())
                    .await
                    .unwrap()
                    .unwrap();

                name = user_query.name;
                email = user_query.email;
            }
        }
    }

    Html(
        format!(
            r#"
            <h1>User info : <br> name - {}, <br> email - {}</h1>
            <form action="http://localhost:3010/logout">
                <input type="submit" value="Logout" />
            </form>
        "#,
            name, email
        )
        .to_string(),
    )
    .into_response()
}

async fn auth(
    Extension(oauth_client): Extension<Arc<Mutex<BasicClient>>>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    cookie_header: Option<TypedHeader<headers::Cookie>>,
) -> impl IntoResponse {
    // Check for an existing session
    if let Some(TypedHeader(ref cookie)) = cookie_header {
        if let Some(session_id) = cookie.get("session_id") {
            // Attempt to convert the session_id to a UUID
            let session_uuid = match Uuid::parse_str(session_id) {
                Ok(uuid) => uuid,
                Err(err) => {
                    println!("Failed to parse session_id as UUID: {}", err);
                    return Html("Invalid session ID format.".to_string()).into_response();
                }
            };

            // Query the database for the session
            let query = entity::session::Entity::find()
                .filter(entity::session::Column::SessionId.eq(session_uuid))
                .one(db.as_ref())
                .await;

            if let Ok(Some(_session)) = query {
                println!("Session found in database: {:?}", _session);
                // Session is valid, proceed without re-authenticating
                return Redirect::temporary("/dashboard").into_response();
            }
        }
    }

    // Generate the OAuth authorization URL and CSRF token
    let (auth_url, _csrf_token) = oauth_client
        .lock()
        .await
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.email".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.profile".to_string(),
        ))
        .url();

    // If session is not found or is invalid, redirect to the OAuth authorization URL
    Redirect::temporary(&auth_url.to_string()).into_response()
}

async fn redirect_auth(
    Query(params): Query<HashMap<String, String>>,
    Extension(oauth_client): Extension<Arc<Mutex<BasicClient>>>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    cookies: Cookies,
) -> impl IntoResponse {

    let state_param = params.get("state").map(|s| s.to_string());

    if let Some(ref state_param) = state_param {
        let csrf_token = CsrfToken::new(state_param.clone());
        println!("Received CSRF token: {}", csrf_token.clone().secret());
    } else {
        return Html("Missing CSRF token".to_string()).into_response();
    }

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

                let url = "https://www.googleapis.com/oauth2/v2/userinfo?oauth_token=".to_owned()
                    + access_token;

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
                        return Html("Email is not verified.".to_string()).into_response();
                    }

                    let query = entity::user::Entity::find().filter(entity::user::Column::Email.eq(email.clone()))
                    .one(db.as_ref())
                    .await;

                    match query {
                        Ok(Some(user)) => {
                            //add all necessary fields into the sessions table.

                            let session_id = Uuid::new_v4();
                            let expires_at_val = Utc::now() + Duration::hours(1);

                            let new_session = entity::session::ActiveModel {
                                session_id: Set(session_id.clone()),
                                user_id: Set(user.uuid),
                                expires_at: Set(expires_at_val.into()),
                                csfr_token: Set(state_param.unwrap_or_else(|| "".to_string())), // Store the CSRF token in the session
                                ..Default::default()
                            };

                            match set_session_id(session_id.to_string()).await {
                                Ok(_) => println!("The session_id was stored in redis"), 
                                Err(_) => eprintln!("The error occured in storing session_id into redis"),
                            }

                            entity::session::Entity::insert(new_session)
                                .exec(db.as_ref())
                                .await
                                .expect("Failed to insert session");

                            //add session_id to Cookies

                            let mut cookie = Cookie::new("session_id", session_id.to_string());
                            cookie.set_http_only(true);
                            cookie.set_path("/");
                            cookie.set_secure(true);
                            cookie.set_same_site(SameSite::Strict);
                            cookies.add(cookie);

                            let body = Body::from(
                                         r#"
                                             <html>
                                             <head>
                                                 <meta http-equiv="refresh" content="0; url=/dashboard" />
                                             </head>
                                             <body>
                                                 User authenticated. Redirecting...
                                             </body>
                                             </html>
                                         "#,
                                     ).into_response();
                            body
                        }
                        Ok(None) => Html("User not found".to_string()).into_response(),
                        Err(err) => {
                            Html(format!("Database query failed: {:?}", err)).into_response()
                        }
                    }
                } else {
                    Html("Failed to parse email from userinfo response".to_string()).into_response()
                }
            }
            Err(err) => {
                eprintln!("Failed to exchange code: {:?}", err);
                Html("Failed to exchange code".to_string()).into_response()
            }
        }
    } else {
        println!("Missing code parameter");
        Html("Missing code".to_string()).into_response()
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

async fn logout(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    cookie_header: Option<TypedHeader<headers::Cookie>>,
) -> impl IntoResponse {
    // Check for an existing session
    if let Some(TypedHeader(ref cookie)) = cookie_header {
        if let Some(session_id) = cookie.get("session_id") {
            // Attempt to convert the session_id to a UUID
            let session_uuid = match Uuid::parse_str(session_id) {
                Ok(uuid) => uuid,
                Err(err) => {
                    println!("Failed to parse session_id as UUID: {}", err);
                    return Html("Invalid session ID format.".to_string()).into_response();
                }
            };
            // Delete the session from the database
            let result = entity::session::Entity::delete_by_id(session_uuid)
                .exec(db.as_ref())
                .await;

            match result {
                Ok(delete_result) => {
                    if delete_result.rows_affected > 0 {
                        println!("Session successfully deleted from the database.");
                    } else {
                        println!("No session found with the given session_id.");
                    }
                }
                Err(err) => {
                    println!("Error deleting session from database: {:?}", err);
                    return Html("Failed to delete session.".to_string()).into_response();
                }
            }

            // Clear the session cookie by setting it with an expiration in the past
            let mut headers = HeaderMap::new();
            headers.insert(
                header::SET_COOKIE,
                "session_id=deleted; HttpOnly; Secure; SameSite=Strict; Max-Age=0"
                    .parse()
                    .unwrap(),
            );

            return (headers, Redirect::temporary("/login")).into_response();
        }
    }

    Redirect::temporary("/login").into_response()
}
