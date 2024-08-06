use std::sync::Arc;

use axum::Router;
use migration::sea_orm::DatabaseConnection;

pub mod user;
pub mod blog;

pub fn create_all_routes(db: Arc<DatabaseConnection>) -> Router {
    Router::new()
        .merge(user::user_routes(db.clone()))
        .merge(blog::blog_routes(db))
}
