use axum::body::Body;
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::response::{Html, Redirect};
use axum::routing::get;
use axum::Extension;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};

use axum_extra::headers;
use axum_extra::TypedHeader;

use chrono::{Duration, Utc};
use migration::sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Set};
use reqwest::header;
use migration::sea_orm::ColumnTrait;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub fn auth_user_routes(db: Arc<DatabaseConnection>) -> Router {
    let oauth_client = create_oauth_client();
    Router::new()
        .route("/auth", get(auth))
        .route("/redirect", get(redirect_auth))
        .route("/dashboard", get(dashboard))
        .route("/login", get(|| async { "Login Page" }))
        .route("/logout", get(logout))
        .layer(Extension(oauth_client))
        .layer(Extension(db))

}

async fn dashboard() -> impl IntoResponse {
    Html(
        r#"
    <form action="http://localhost:3010/logout">
        <input type="submit" value="Logout" />
    </form>
"#
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
        .add_scope(Scope::new("openid".to_string()))
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
                //let refresh_token = token.refresh_token().unwrap().secret();

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

                    let query = entity::user::Entity::find()
                        .filter(entity::user::Column::Email.eq(email.clone()))
                        .one(db.as_ref())
                        .await;

                    match query {
                        Ok(Some(user)) => {
                            //add all necessary fields into the sessions table.

                            let session_id = Uuid::new_v4();
                            let expires_at_val = Utc::now() + Duration::hours(1);

                            let new_session = entity::session::ActiveModel {
                                session_id: Set(session_id.clone()),
                                user_id: Set(user.id),
                                access_token: Set(access_token.clone()),
                                refresh_token: Set("".to_string()),
                                expires_at: Set(expires_at_val.into()),
                                csfr_token: Set(state_param.unwrap_or_else(|| "".to_string())), // Store the CSRF token in the session
                                ..Default::default()
                            };

                            entity::session::Entity::insert(new_session)
                                .exec(db.as_ref())
                                .await
                                .expect("Failed to insert session");

                            //add session_id to Cookies
                            let mut headers = HeaderMap::new();
                            headers.insert(
                                header::SET_COOKIE,
                                format!(
                                    "session_id={}; HttpOnly; Secure; SameSite=Strict",
                                    session_id
                                )
                                .parse()
                                .unwrap(),
                            );
                            // Include the Authorization header with the Bearer token
                            headers.insert(
                                header::AUTHORIZATION,
                                format!("Bearer {}", session_id).parse().unwrap(),
                            );

                            Response::builder()
                                .status(StatusCode::OK)
                                .header(header::AUTHORIZATION, format!("Bearer {}", session_id))
                                .header(
                                    header::SET_COOKIE,
                                    format!(
                                        "session_id={}; HttpOnly; Secure; SameSite=Strict",
                                        session_id
                                    ),
                                )
                                .body(Body::from(
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
                                ))
                                .unwrap()
                                .into_response()
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
