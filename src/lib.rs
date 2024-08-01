use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;
mod routes;
mod models;


pub async fn run(db : Arc<DatabaseConnection>) {
    let app = routes::create_all_routes(db);

    let listener = TcpListener::bind("127.0.0.1:3010")
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}