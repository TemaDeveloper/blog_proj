mod models;
use std::{env, sync::Arc};

use blog_proj::run;
use dotenv::dotenv;
use migration::sea_orm::Database;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //init dotenv
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_conn = Database::connect(&db_url).await?;
    let db_conn = Arc::new(db_conn);

    run(db_conn).await;

    Ok(())
}

