use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Serialize, Deserialize, Clone)]
pub struct CreateBlogModel{
    pub title : String, 
    pub content : String, 
    pub user_id : Uuid,
    pub images : Option<Vec<String>>,
}


#[derive(Serialize, Deserialize)]
pub struct UpdateBlogModel{
    pub title : String, 
    pub content : String,
}

#[derive(Serialize, Deserialize)]
pub struct GetBlogModel{
    pub title : String, 
    pub content : String, 
    pub user_id : Uuid,
    pub created_at : DateTime<FixedOffset>,
    pub images : Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct GetAllBlogsModel{
    pub blogs : Vec<GetBlogModel>
}
