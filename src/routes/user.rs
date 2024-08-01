use axum::routing::get;
use axum::Extension;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use entity::user;
use models::user_models::{GetAllUsersModel, UserModelPub, CreateUserModel};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use std::sync::Arc;
use uuid::Uuid;
use crate::models;

pub fn user_routes(db: Arc<DatabaseConnection>) -> Router {
    Router::new()
        .route("/users", get(get_all_users).layer(Extension(db.clone())))
        .route("/user/insert", post(create_user).layer(Extension(db)))
} 

async fn get_all_users(Extension(db): Extension<Arc<DatabaseConnection>>) -> impl IntoResponse {
    
    let users = entity::user::Entity::find().all(db.as_ref()).await;

    match users {
        Ok(res) => (
            StatusCode::OK,
            Json(GetAllUsersModel {
                users: res
                    .iter()
                    .map(|u| UserModelPub {
                        name: (*u.name).to_string(),
                        email: (*u.email).to_string(),
                    })
                    .collect(),
            }),
        ),
        Err(_) => (StatusCode::NOT_FOUND, Json::default()),
    }
}

async fn create_user(Extension(db): Extension<Arc<DatabaseConnection>>, user_data: Json<CreateUserModel>) -> impl IntoResponse {


    let user_model = user::ActiveModel {
        name: ActiveValue::Set(user_data.name.to_owned()),
        email: ActiveValue::Set(user_data.email.to_owned()),
        password: ActiveValue::Set(user_data.password.to_owned()),
        uuid: ActiveValue::Set(Uuid::new_v4()),
        ..Default::default()
    };

    user_model.clone().insert(db.as_ref()).await.unwrap();

    //db.close().await.unwrap();
    (StatusCode::ACCEPTED, format!("{:?}", user_model.uuid))
}
