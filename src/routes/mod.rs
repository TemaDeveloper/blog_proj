use std::sync::Arc;

use auth::auth_user_routes;
use axum::{routing::get_service, Router};
use migration::sea_orm::DatabaseConnection;
use registration::register_routing;
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::{AllowHeaders, CorsLayer}, services::ServeDir};
use http::{HeaderValue, Method};

pub mod user;
pub mod blog;
pub mod auth;
pub mod registration;
pub mod middlewares;
pub mod file_upload;



pub async fn create_all_routes(db: Arc<DatabaseConnection>) -> Router {

    let cors = CorsLayer::new()
        .allow_methods([Method::POST, Method::GET, Method::PUT, Method::DELETE])
        .allow_origin("http://localhost:3010".parse::<HeaderValue>().unwrap())
        .allow_credentials(true)
        .allow_headers(AllowHeaders::list(vec![
            "Content-Type".parse().unwrap(),
            "Authorization".parse().unwrap(),
        ]));

    Router::new()
        .merge(auth_user_routes(db.clone()))
        .merge(register_routing(db.clone()))
        .merge(user::user_routes(db.clone()))
        .merge(blog::blog_routes(db))
        .merge(file_upload::upload_router().await)
        .layer(cors)
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static())       
}

fn routes_static() -> Router {
	Router::new().nest_service("/", get_service(ServeDir::new("./")))
}