use axum::body::Body;
use axum::extract::Path;
use axum::routing::{get, put};
use axum::{Extension, Form};
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
use tower_cookies::cookie::SameSite;
use tower_cookies::{Cookie, Cookies};
use crate::models;
use crate::models::user_models::{GetUserModel, UpdateUserModel};
use std::sync::Arc;
use uuid::Uuid;

pub fn user_routes(db: Arc<DatabaseConnection>) -> Router {

    Router::new()
        .route("/users", get(get_all_users))
        .route("/user/insert", post(sign_on_by_credentials))
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
    Path(id) : Path<Uuid>,
)
-> impl IntoResponse{

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
            uuid : user.uuid, 
        })
    )
} 

async fn update_user(
    Extension(db) : Extension<Arc<DatabaseConnection>>,
    Path(id) : Path<Uuid>,
    updated_user : Json<UpdateUserModel>
) -> impl IntoResponse{

    let mut user : entity::user::ActiveModel = entity::user::Entity::find()
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

//TODO: Add Password encoding and email verification

async fn sign_on_by_credentials(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    cookies : Cookies,
    user_data: Form<CreateUserModel>,
    
) -> impl IntoResponse {
    let user_model = user::ActiveModel {
        name: Set(user_data.name.to_owned()),
        email: Set(user_data.email.to_owned()),
        password: Set(user_data.password.to_owned()),
        uuid: Set(Uuid::new_v4()),
        ..Default::default()
    };

    let session_id = Uuid::new_v4();

    let mut cookie = Cookie::new("session_id", session_id.to_string());
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::Strict);
    cookies.add(cookie);

    // let new_session = entity::session::ActiveModel {
    //     session_id: Set(session_id.clone()),
    //     user_id: Set(user_model.uuid.unwrap()),
    //     access_token: Set(access_token.clone()),
    //     refresh_token: Set("".to_string()),
    //     expires_at: Set(expires_at_val.into()),
    //     csfr_token: Set(state_param.unwrap_or_else(|| "".to_string())), // Store the CSRF token in the session
    //     ..Default::default()
    // };

    // entity::session::Entity::insert(new_session)
    //     .exec(db.as_ref())
    //     .await
    //     .expect("Failed to insert session");

    user::Entity::insert(user_model.clone())
        .exec(db.as_ref())
        .await
        .unwrap();

        let body = Body::from(
            r#"
                <html>
                <head>
                    <meta http-equiv="refresh" content="0; url=/dashboard" />
                </head>
                <body>
                    User authenticated. Redirecting...
                </body>
                </html>
            "#,
        ).into_response();

        (StatusCode::CREATED, body)

}

//TODO: Upload Image
