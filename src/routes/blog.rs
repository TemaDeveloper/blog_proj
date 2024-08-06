use crate::models::blog_model::{CreateBlogModel, GetAllBlogsModel, GetBlogModel, UpdateBlogModel};
use axum::extract::Path;
use axum::routing::{delete, get};
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{post, put},
    Extension, Json, Router,
};
use entity::{blog, user};
use migration::sea_orm::ColumnTrait;
use migration::sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

pub fn blog_routes(db: Arc<DatabaseConnection>) -> Router {
    Router::new()
        .route("/blog/insert", post(create_blog))
        .route("/blog/update/:id", put(update_blog))
        .route("/blog/delete/:id", delete(delete_blog))
        .route("/blog/:id", get(get_blog))
        .route("/blogs", get(get_all_blogs))
        .route("/blogs/user/:id", get(get_all_user_blogs))
        .layer(Extension(db))
}

async fn get_all_user_blogs(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> impl IntoResponse {
    let blogs = entity::blog::Entity::find()
        .filter(blog::Column::UserId.eq(id))
        .all(db.as_ref())
        .await;

    match blogs {
        Ok(res) => (
            StatusCode::OK,
            Json(GetAllBlogsModel {
                blogs: res
                    .iter()
                    .map(|b| GetBlogModel {
                        title: (*b.title).to_string(),
                        content: (*b.content).to_string(),
                        user_id: b.user_id,
                        created_at: b.created_at,
                    })
                    .collect(),
            }),
        ),
        Err(_) => (StatusCode::NOT_FOUND, Json::default()),
    }
}

async fn get_all_blogs(Extension(db): Extension<Arc<DatabaseConnection>>) -> impl IntoResponse {
    //extract all blogs from db
    let blogs = entity::blog::Entity::find().all(db.as_ref()).await;

    //match the vector of blogs
    match blogs {
        Ok(res) => (
            StatusCode::OK,
            //if the result is Ok return Json with the Vector of users
            Json(GetAllBlogsModel {
                //iterate each blog
                blogs: res
                    .iter()
                    //map each model of blog with its arguments
                    .map(|b| GetBlogModel {
                        title: (*b.title).to_string(),
                        content: (*b.content).to_string(),
                        user_id: b.user_id,
                        created_at: b.created_at,
                    })
                    .collect(),
            }),
        ),
        Err(_) => (StatusCode::NOT_FOUND, Json::default()),
    }
}

async fn get_blog(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> impl IntoResponse {
    let blog = entity::blog::Entity::find()
        .filter(entity::blog::Column::Id.eq(id))
        .one(db.as_ref())
        .await
        .unwrap()
        .unwrap();

    (
        StatusCode::OK,
        Json(GetBlogModel {
            title: blog.title,
            content: blog.content,
            user_id: blog.user_id,
            created_at: blog.created_at,
        }),
    )
}

//delete blog by its id
async fn delete_blog(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> impl IntoResponse {
    let blog = entity::blog::Entity::find()
        .filter(entity::blog::Column::Id.eq(id))
        .one(db.as_ref())
        .await
        .unwrap()
        .unwrap();

    entity::blog::Entity::delete_by_id(blog.id)
        .exec(db.as_ref())
        .await
        .unwrap();

    //db.close().await.unwrap();

    (StatusCode::ACCEPTED, "Deleted")
}

async fn update_blog(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    Json(blog_data): Json<UpdateBlogModel>,
) -> impl IntoResponse {
    //dotenv().ok();
    //let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    //let db_conn = Database::connect(&db_url).await.unwrap();

    let mut blog: entity::blog::ActiveModel = entity::blog::Entity::find()
        .filter(entity::blog::Column::Id.eq(id))
        .one(db.as_ref())
        .await
        .unwrap()
        .unwrap()
        .into();

    blog.title = Set(blog_data.title);
    blog.content = Set(blog_data.content);

    blog::Entity::update(blog).exec(db.as_ref()).await.unwrap();

    (StatusCode::ACCEPTED, "Updated")
}

async fn create_blog(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    blog_data: Json<CreateBlogModel>,
) -> impl IntoResponse {
    //* Refactored :) */
    // if the user's id (PRIMARY KEY) == user_id that is given as argument => insert the new blog
    // Check if user exists
    match user::Entity::find()
        .filter(user::Column::Id.eq(blog_data.user_id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(user)) => {
            println!("User found: {:?}", user);

            // User exists, insert blog
            let blog_model = blog::ActiveModel {
                title: Set(blog_data.title.to_owned()),
                content: Set(blog_data.content.to_owned()),
                user_id: Set(blog_data.user_id),
                image: Set("image/file route".to_string()),
                ..Default::default()
            };

            // Insertion to DB
            match blog::Entity::insert(blog_model.clone())
                .exec(db.as_ref())
                .await
            {
                Ok(_) => (StatusCode::CREATED, format!("{:?}", blog_model)).into_response(),
                Err(e) => {
                    eprintln!("Database insertion error: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to insert blog".to_string(),
                    )
                        .into_response()
                }
            }
        }
        Ok(None) => {
            // User not found
            (StatusCode::FORBIDDEN, "You have no rights".to_string()).into_response()
        }
        Err(e) => {
            eprintln!("Database query error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query user".to_string(),
            )
                .into_response()
        }
    }
}
