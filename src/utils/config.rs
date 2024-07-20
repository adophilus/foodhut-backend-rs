use std::env;

#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u32,
    pub url: String,
}

#[derive(Clone)]
pub struct StorageConfig {
    pub api_key: String,
    pub api_secret: String,
    pub upload_endpoint: String,
    pub delete_endpoint: String,
    pub upload_preset: String,
}

#[derive(Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub app: AppConfig,
    pub storage: StorageConfig,
}

pub fn get_config() -> Config {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u32>()
        .expect("Invalid PORT number");
    let url = env::var("URL").unwrap_or_else(|_| format!("http://{}:{}", host, port));
    let storage_api_key = env::var("CLOUDINARY_API_KEY").expect("CLOUDINARY_API_KEY not set");
    let storage_api_secret =
        env::var("CLOUDINARY_API_SECRET").expect("CLOUDINARY_API_SECRET not set");
    let storage_upload_endpoint =
        env::var("CLOUDINARY_UPLOAD_ENDPOINT").expect("CLOUDINARY_UPLOAD_ENDPOINT not set");
    let storage_delete_endpoint =
        env::var("CLOUDINARY_DELETE_ENDPOINT").expect("CLOUDINARY_DELETE_ENDPOINT not set");
    let storage_upload_preset =
        env::var("CLOUDINARY_UPLOAD_PRESET").expect("CLOUDINARY_UPLOAD_PRESET not set");

    return Config {
        database: DatabaseConfig { url: database_url },
        app: AppConfig { host, port, url },
        storage: StorageConfig {
            api_key: storage_api_key,
            api_secret: storage_api_secret,
            upload_endpoint: storage_upload_endpoint,
            delete_endpoint: storage_delete_endpoint,
            upload_preset: storage_upload_preset,
        },
    };
}
