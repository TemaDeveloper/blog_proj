// use std::sync::Arc;

// use axum::{body::Body, middleware::Next, response::IntoResponse, Extension};
// use axum_extra::{headers, TypedHeader};
// use http::Request;
// use migration::sea_orm::DatabaseConnection;


// pub async fn inject_user_data(
//     Extension(db) : Extension<Arc<DatabaseConnection>>,
//     cookie : Option<TypedHeader<headers::Cookie>>, 
//     mut request : Request<Body>, 
//     next : Next,
// ) -> impl IntoResponse {

//     if let Some(cookie) = cookie {
//         if let Some(session_id) = cookie.get("session_id"){}
//     }

// }
