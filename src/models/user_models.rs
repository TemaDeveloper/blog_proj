use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Deserialize, Serialize, Clone)]
pub struct UserModel{
    pub name : String, 
    pub email : String, 
    pub uuid : Uuid, 
}

#[derive(Deserialize, Serialize)]
pub struct UpdateUserModel{
    pub name : String,
}


#[derive(Deserialize, Serialize)]
pub struct GetUserModel{
    pub name : String, 
    pub email : String, 
    pub id : i32,
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
}



