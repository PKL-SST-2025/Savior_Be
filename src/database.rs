use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

pub type Database = PgPool;

pub async fn create_database_connection() -> Result<Database, sqlx::Error> {
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    println!("✅ Database connected successfully");
    Ok(pool)
}

pub async fn run_migrations(pool: &Database) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(pool).await?;
    println!("✅ Migrations executed successfully");
    Ok(())
}
