use std::env;

#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: String,
}

#[derive(Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub app: AppConfig,
}

pub fn get_config() -> Config {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());

    return Config {
        database: DatabaseConfig { url: database_url },
        app: AppConfig {
            host,
            port,
        },
    };
}
