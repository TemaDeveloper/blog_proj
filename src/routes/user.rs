use axum::routing::get;
use axum::Extension;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};

use entity::user;
use migration::sea_orm::{DatabaseConnection, EntityTrait, Set};
use models::user_models::{CreateUserModel, GetAllUsersModel, UserModelPub};
use crate::models;
use std::sync::Arc;
use uuid::Uuid;



pub fn user_routes(db: Arc<DatabaseConnection>) -> Router {
    //tenant - dev-osgutng6i3uptora
    //domain - dev-osgutng6i3uptora.auth0.com

    Router::new()
        .route("/users", get(get_all_users))
        .route("/user/insert", post(create_user))
        .route("/privacy", get(|| async { "Privacy Policy" }))
        .route("/tos", get(|| async { "TOS" }))
        .layer(Extension(db))
       // .merge(auth_user_routes())
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

async fn create_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    user_data: Json<CreateUserModel>,
) -> impl IntoResponse {
    let user_model = user::ActiveModel {
        name: Set(user_data.name.to_owned()),
        email: Set(user_data.email.to_owned()),
        uuid: Set(Uuid::new_v4()),
        ..Default::default()
    };

    user::Entity::insert(user_model.clone())
        .exec(db.as_ref())
        .await
        .unwrap();

    (StatusCode::CREATED, format!("{:?}", user_model.uuid))
}

//TODO: Upload Image
