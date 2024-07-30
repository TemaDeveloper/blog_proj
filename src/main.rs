use std::env;
use dotenv::dotenv;
use axum::{http::StatusCode, response::IntoResponse, routing::{get, post}, Router};
use sea_orm::{ActiveModelTrait, ActiveValue, Database};
use tokio::net::TcpListener;
use uuid::Uuid;

use entity::user;

#[tokio::main]
async fn main() {


    let routes = Router::new()
        .route("/user/test/insert", get(create_user));

    let listener = TcpListener::bind("127.0.0.1:3010")
        .await
        .unwrap();

    axum::serve(listener, routes).await.unwrap();
}

async fn create_user() -> impl IntoResponse{

    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Database::connect(&db_url).await.unwrap();
    let user_model = user::ActiveModel{
        name: ActiveValue::Set("TestName".to_string()),
        email: ActiveValue::Set("NewMailTest12".to_string()),
        password: ActiveValue::Set("TestPass".to_string()),
        uuid: ActiveValue::Set(Uuid::new_v4()), 
        ..Default::default()
    };

    let usr = user_model.insert(&db).await.unwrap();

    (StatusCode::ACCEPTED, "inserted")

}
