use axum::extract::Path;
use axum::routing::{get, put};
use axum::Extension;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use migration::sea_orm::ColumnTrait;
use entity::user;
use migration::sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Set};
use models::user_models::{CreateUserModel, GetAllUsersModel, UserModelPub};
use crate::models;
use crate::models::user_models::{GetUserModel, UpdateUserModel};
use std::sync::Arc;
use uuid::Uuid;

pub fn user_routes(db: Arc<DatabaseConnection>) -> Router {

    Router::new()
        .route("/users", get(get_all_users))
        .route("/user/insert", post(create_user))
        .route("/privacy", get(|| async { "Privacy Policy" }))
        .route("/tos", get(|| async { "TOS" }))
        .route("/user/:id", get(get_user))
        .route("/user/update/:id", put(update_user))
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
    Extension(db) : Extension<Arc<DatabaseConnection>>, 
    Path(id) : Path<i32>,
)
-> impl IntoResponse{

    let user = entity::user::Entity::find()
        .filter(entity::user::Column::Id.eq(id))
        .one(db.as_ref())
        .await
        .unwrap()
        .unwrap();
    (
        StatusCode::OK, 
        Json(GetUserModel {
            name: user.name.to_string(),
            email: user.email.to_string(),
            id : user.id, 
        })
    )
} 

async fn update_user(
    Path(id) : Path<i32>,
    Extension(db) : Extension<Arc<DatabaseConnection>>,
    updated_user : Json<UpdateUserModel>
) -> impl IntoResponse{

    let mut user : entity::user::ActiveModel = entity::user::Entity::find()
        .filter(entity::user::Column::Id.eq(id))
        .one(db.as_ref())
        .await
        .unwrap()
        .unwrap()
        .into();

    user.name = Set(updated_user.name.clone());

    user::Entity::update(user).exec(db.as_ref()).await.unwrap();

    (StatusCode::ACCEPTED, "Updated")

}

//TODO: Make the registration, using Oauth2, using /userinfo get the information from endpoint

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
