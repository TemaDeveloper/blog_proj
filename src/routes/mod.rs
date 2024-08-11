use std::sync::Arc;

use auth::auth_user_routes;
use axum::Router;
use migration::sea_orm::DatabaseConnection;

pub mod user;
pub mod blog;
pub mod auth;

pub fn create_all_routes(db: Arc<DatabaseConnection>) -> Router {
    Router::new()
        .merge(auth_user_routes(db.clone()))
        .merge(user::user_routes(db.clone()))
        .merge(blog::blog_routes(db))
        
}
