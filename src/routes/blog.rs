use axum::extract::Path;
use axum::routing::{delete, get};
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{post, put},
    Json, Router,
    Extension,
};
use entity::blog;
use crate::models::blog_model::{GetAllBlogsModel, GetBlogModel, UpdateBlogModel, CreateBlogModel};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

pub fn blog_routes(db: Arc<DatabaseConnection>) -> Router {
    Router::new()
        .route("/blog/insert", post(create_blog).layer(Extension(db.clone())))
        .route("/blog/update/:id", put(update_blog).layer(Extension(db.clone())))
        .route("/blog/delete/:id", delete(delete_blog).layer(Extension(db.clone())))
        .route("/blog/:id", get(get_blog).layer(Extension(db.clone())))
        .route("/blogs", get(get_all_blogs).layer(Extension(db.clone())))
        .route("/blogs/user/:id", get(get_all_user_blogs).layer(Extension(db)))
}

async fn get_all_user_blogs(
    Path(id): Path<i32>, 
    Extension(db) : Extension<Arc<DatabaseConnection>>,
) -> impl IntoResponse {

    let blogs = entity::blog::Entity::find()
        .filter(entity::blog::Column::UserId.eq(id))
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


async fn get_all_blogs(
    Extension(db) : Extension<Arc<DatabaseConnection>>
) -> impl IntoResponse {
    
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

async fn get_blog(Path(id): Path<i32>, Extension(db) : Extension<Arc<DatabaseConnection>>) -> impl IntoResponse {


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
async fn delete_blog(Path(id): Path<i32>, Extension(db): Extension<Arc<DatabaseConnection>>) -> impl IntoResponse {
    

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

    blog.update(db.as_ref()).await.unwrap();

    (StatusCode::ACCEPTED, "Updated")
}

async fn create_blog(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    blog_data: Json<CreateBlogModel>, 
) -> impl IntoResponse {
   
    //dotenv().ok();
    //let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    //let db_conn = Database::connect(&db_url).await.unwrap();

    // if the user's id (PRIMARY KEY) == user_id that is given as argument => insert the new blog

    if entity::user::Entity::find()
        .filter(Condition::all().add(entity::user::Column::Id.eq(blog_data.user_id)))
        .one(db.as_ref())
        .await
        .unwrap()
        .is_some()
    {
        // if the column id == user_id and there is a found model of this user then insert

        let blog_model = blog::ActiveModel {
            title: ActiveValue::Set(blog_data.title.to_owned()),
            content: ActiveValue::Set(blog_data.content.to_owned()),
            user_id: ActiveValue::Set(blog_data.user_id.to_owned()),
            ..Default::default()
        };

        //insertion to DB
        blog_model.clone().insert(db.as_ref()).await.unwrap();

        //db_conn.close().await.unwrap();
        return (StatusCode::CREATED, format!("{:?}", blog_model));
    } else {
        return (StatusCode::FORBIDDEN, "You have no rights".to_string());
    }
}

