use std::sync::Arc;
use axum::{
    body::Body, 
    middleware::Next, 
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use migration::sea_orm::ColumnTrait;

use axum_extra::{headers, TypedHeader};
use chrono::Utc;
use entity::session;
use http::{header, Request, StatusCode};
use migration::sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

pub async fn user_expired(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    cookie: Option<TypedHeader<headers::Cookie>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(cookie) = cookie {
        if let Some(session_id) = cookie.get("session_id") {

            let session_id = match session_id.parse::<Uuid>() {
                Ok(uuid) => uuid,
                Err(_) => {
                    println!("Failed to parse session_id as UUID");
                    return Ok((StatusCode::BAD_REQUEST, "Invalid session ID").into_response());
                }
            };

            let session = match entity::session::Entity::find()
                .filter(session::Column::SessionId.eq(session_id))
                .one(db.as_ref())
                .await
            {
                Ok(Some(session)) => session,
                Ok(None) => return Ok((StatusCode::UNAUTHORIZED,  Redirect::temporary("/login")).into_response()),
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            };

            println!("Expires_at : {:?}", session.expires_at);

            if session.expires_at < Utc::now() {
                let result = entity::session::Entity::delete_by_id(session.session_id)
                    .exec(db.as_ref())
                    .await;

                match result {
                    Ok(delete_result) => {
                        if delete_result.rows_affected > 0 {
                            println!("Session successfully deleted from the database.");
                            Redirect::temporary("/login").into_response();
                        } else {
                            println!("No session found with the given session_id.");
                        }
                    }
                    Err(err) => {
                        println!("Error deleting session from database: {:?}", err);
                    }
                }

                // Clear the session cookie by setting it with an expiration in the past
                let mut response = (StatusCode::UNAUTHORIZED, Redirect::temporary("/login")).into_response();
                let headers = response.headers_mut();
                headers.insert(
                    header::SET_COOKIE,
                    "session_id=deleted; HttpOnly; Secure; SameSite=Strict; Max-Age=0"
                        .parse()
                        .unwrap(),
                );
                return Ok(response);
            }
        }
    }

    // Proceed with the next middleware or handler if the session is still valid
    Ok(next.run(request).await)
}
