use std::sync::Arc;

use migration::sea_orm::DatabaseConnection;
use tokio::net::TcpListener;
mod routes;
mod models;
mod redis_manager;


pub async fn run(db : Arc<DatabaseConnection>) {
    let app = routes::create_all_routes(db);

    let listener = TcpListener::bind("localhost:3010")
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}