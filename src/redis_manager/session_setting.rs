use dotenv::dotenv;
use redis::{AsyncCommands, Commands};
use std::env;

//TODO: why it is not used? 

pub async fn set_session_id(uuid : String) -> redis::RedisResult<isize> {
    dotenv().ok();
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = redis::Client::open(redis_url)?;
    let mut con = client.get_connection().expect("Connection with redis is not working :(");
    con.set_ex(&uuid, 1, 3600)
}

pub async fn get_session_id(uuid : String) -> redis::RedisResult<bool>{
    dotenv().ok();
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = redis::Client::open(redis_url)?;
    let mut con = client.get_multiplexed_async_connection().await?;
    let exists: bool = con.exists(&uuid).await?;

    Ok(exists)
}