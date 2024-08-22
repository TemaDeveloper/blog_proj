use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Body,
    extract::Query,
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Extension, Router,
};
use chrono::{Duration, Utc};
use entity::user;
use migration::sea_orm::{DatabaseConnection, EntityTrait, Set};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use std::env;
use tokio::sync::Mutex;
use tower_cookies::{cookie::SameSite, Cookie, Cookies};
use uuid::Uuid;

use crate::redis_manager::session_setting::set_session_id;


pub fn register_routing(db: Arc<DatabaseConnection>) -> Router {
    let oauth_client = create_oauth_client();

    Router::new()
        .route("/auth_sign_on", get(auth_registration))
        .route("/register_redirect", get(redirect_sign_on))
        .layer(Extension(oauth_client))
        .layer(Extension(db))
}

async fn auth_registration(
    Extension(oauth_client): Extension<Arc<Mutex<BasicClient>>>,
) -> impl IntoResponse {
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

fn create_oauth_client() -> Arc<Mutex<BasicClient>> {
    let client_id = ClientId::new(env::var("GOOGLE_OAUTH_CLIENT_ID").expect("Missing CLIENT_ID"));
    let client_secret =
        ClientSecret::new(env::var("GOOGLE_OAUTH_CLIENT_SECRET").expect("Missing CLIENT_SECRET"));
    let auth_url = AuthUrl::new(env::var("OAUTH_AUTH_URL").expect("Missing AUTH_URL"))
        .expect("Invalid AUTH_URL");
    let token_url = TokenUrl::new(env::var("OAUTH_TOKEN_URL").expect("Missing TOKEN_URL"))
        .expect("Invalid TOKEN_URL");
    let redirect_url =
        RedirectUrl::new(env::var("OAUTH_REDIRECT_SIGN_ON_URL").expect("Missing REDIRECT_URL"))
            .expect("Invalid REDIRECT_URL");
    let oauth_client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url);

    Arc::new(Mutex::new(oauth_client))
}

pub async fn redirect_sign_on(
    Query(params): Query<HashMap<String, String>>,
    Extension(oauth_client): Extension<Arc<Mutex<BasicClient>>>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    cookies: Cookies,
) -> impl IntoResponse {
    // Extract CSRF token (state parameter) from the OAuth provider response
    let state_param = params.get("state").map(|s| s.to_string());

    // Validate the CSRF token
    if let Some(ref state_param) = state_param {
        let csrf_token = CsrfToken::new(state_param.clone());
        // Optionally compare this with a stored value or handle it securely
        println!("Received CSRF token: {}", csrf_token.clone().secret());
        // Continue with your authentication logic
    } else {
        // Missing CSRF token in the OAuth response
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

                // Use Bearer token in the Authorization header
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

                let session_id = Uuid::new_v4();
                let expires_at_val = Utc::now() + Duration::hours(1);

                let name_resource_server = body["name"].as_str().unwrap();
                let email_resource_server = body["email"].as_str().unwrap();


                    let user_model = user::ActiveModel {
                        name: Set(name_resource_server.to_string()),
                        email: Set(email_resource_server.to_string()),
                        uuid: Set(Uuid::new_v4()),
                        ..Default::default()
                    };

                    user::Entity::insert(user_model.clone())
                        .exec(db.as_ref())
                        .await
                        .unwrap();

                    let new_session = entity::session::ActiveModel {
                        session_id: Set(session_id.clone()),
                        user_id: Set(user_model.uuid.unwrap()),
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
            Err(err) => Html(format!("Database query failed: {:?}", err)).into_response(),
        }
    } else {
        Html("Failed to parse email from userinfo response".to_string()).into_response()
    }
}
