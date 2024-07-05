use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Clone)]
pub struct DatabaseConnection{
    pub pool: PgPool,
}

pub async fn connect(database_url: &str) -> DatabaseConnection {
    DatabaseConnection {
        pool: PgPoolOptions::new()
            .max_connections(4)
            .connect(database_url)
            .await
            .unwrap_or_else(|_| panic!("Error connecting to database {}", database_url)),
    }
}
