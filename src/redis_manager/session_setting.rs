use dotenv::dotenv;
use redis::{AsyncCommands, Client};
use std::{env, error::Error};

pub async fn set_session_id(uuid : String) -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = Client::open(redis_url)?;
    let mut con = client.get_multiplexed_tokio_connection().await?;
    let session_set = con.set_ex(&uuid, 1, 3600).await?;
    Ok(session_set)
}

pub async fn get_session_id(uuid : String) -> redis::RedisResult<bool>{
    dotenv().ok();
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = Client::open(redis_url)?;
    let mut con = client.get_multiplexed_tokio_connection().await?;
    let exists: bool = con.exists(&uuid).await?;
    
    Ok(exists)
}