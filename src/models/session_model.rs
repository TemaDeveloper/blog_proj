use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Serialize, Debug)]
pub struct CreateUserSession {
    csrf_token : String, 
    refresh_token : String, 
    access_token : String,
    user_id : i32,
} 

#[derive(Deserialize, Debug)]
pub struct GetUserSession{
    session_id : Uuid, 
    expires_at : DateTime<FixedOffset>, 
    csrf_token : String, 
    refresh_token : String, 
    access_token : String,
    user_id : i32,
}