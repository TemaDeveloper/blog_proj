use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone)]
pub struct CreateBlogModel{
    pub title : String, 
    pub content : String, 
    pub user_id : i32,
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
    pub user_id : i32,
    pub created_at : DateTime<FixedOffset>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct GetAllBlogsModel{
    pub blogs : Vec<GetBlogModel>
}
