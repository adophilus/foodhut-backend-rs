use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Clone)]
pub struct DatabaseConnection {
    pub pool: PgPool,
}

pub async fn connect(database_url: &str) -> DatabaseConnection {
    DatabaseConnection {
        pool: PgPoolOptions::new()
            .max_connections(4)
            .connect(database_url)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("{:}", e);
                panic!("Error connecting to database {}", database_url)
            }),
    }
}

pub async fn migrate(db_conn: DatabaseConnection) {
    match sqlx::migrate!().run(&db_conn.pool).await {
        Ok(_) => (),
        Err(err) => {
            tracing::error!("{}", err);
            panic!("Failed to run database migrations");
        }
    }
}
