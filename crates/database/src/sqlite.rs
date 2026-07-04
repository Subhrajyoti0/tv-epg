use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};

pub type DbPool = Pool<Sqlite>;

pub async fn connect(database_url: &str) -> anyhow::Result<DbPool> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(20)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn migrate(pool: &DbPool) -> anyhow::Result<()> {
    let sql = include_str!("../../../migrations/0001_initial.sql");

    for statement in sql.split(';') {
        let statement = statement.trim();

        if !statement.is_empty() {
            sqlx::query(statement).execute(pool).await?;
        }
    }

    Ok(())
}
