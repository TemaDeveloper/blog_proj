use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Deserialize, Serialize, Clone)]
pub struct UserModel{
    pub name : String, 
    pub email : String, 
    pub password : String, 
    pub uuid : Uuid, 
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UserModelPub{
    pub name : String, 
    pub email : String, 
}


#[derive(Deserialize, Serialize, Default)]

pub struct GetAllUsersModel{
    pub users : Vec<UserModelPub>
}


#[derive(Deserialize, Serialize)]
pub struct CreateUserModel{
    pub name : String, 
    pub email : String, 
    pub password : String,
}

#[derive(Deserialize, Serialize)]
pub struct LoginUserModel{
    pub email : String, 
    pub password : String,
}

