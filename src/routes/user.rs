use crate::models;
use crate::models::user_models::{CreateUserModel, GetUserModel, UpdateUserModel};
use axum::extract::Path;
use axum::routing::{get, post, put};
use axum::{http::StatusCode, response::IntoResponse, Json, Router};
use axum::Extension;
use entity::user;
use migration::sea_orm::ColumnTrait;
use migration::sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Set};
use models::user_models::{GetAllUsersModel, UserModelPub};
use std::sync::Arc;
use uuid::Uuid;

pub fn user_routes(db: Arc<DatabaseConnection>) -> Router {
    Router::new()
        .route("/users", get(get_all_users))
        .route("/privacy", get(|| async { "Privacy Policy" }))
        .route("/tos", get(|| async { "TOS" }))
        .route("/user/:id", get(get_user))
        .route("/user/update/:id", put(update_user))
        .route("/user/insert", post(register_user))
        .layer(Extension(db))
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

async fn get_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user = entity::user::Entity::find()
        .filter(entity::user::Column::Uuid.eq(id))
        .one(db.as_ref())
        .await
        .unwrap()
        .unwrap();
    (
        StatusCode::OK,
        Json(GetUserModel {
            name: user.name.to_string(),
            email: user.email.to_string(),
            uuid: user.uuid,
        }),
    )
}

async fn update_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    Path(id): Path<Uuid>,
    updated_user: Json<UpdateUserModel>,
) -> impl IntoResponse {
    let mut user: entity::user::ActiveModel = entity::user::Entity::find()
        .filter(entity::user::Column::Uuid.eq(id))
        .one(db.as_ref())
        .await
        .unwrap()
        .unwrap()
        .into();

    user.name = Set(updated_user.name.clone());

    user::Entity::update(user).exec(db.as_ref()).await.unwrap();

    (StatusCode::ACCEPTED, "Updated")
}

async fn register_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    user_data : Json<CreateUserModel>,
) -> impl IntoResponse {

    let user_id = Uuid::new_v4();

    let user_model = entity::user::ActiveModel{
        name: Set(user_data.name.to_owned()),
        email: Set(user_data.email.to_owned()),
        uuid: Set(user_id),
    };
        
    let new_user = entity::user::Entity::insert(user_model)
        .exec(db.as_ref())
        .await;

    match new_user {
        Ok(user) => (StatusCode::CREATED, format!("User was created successfully - > user : {:?}", user)), 
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, format!("The error occured! :("))
    }

}

//TODO: Upload Image
