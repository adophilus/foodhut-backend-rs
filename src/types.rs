pub use crate::utils::database;
use async_convert::{async_trait, TryFrom};
use std::env;

#[derive(Clone)]
pub struct AppContext {
    pub host: String,
    pub port: u32,
    pub url: String,
}

#[derive(Clone)]
pub struct StorageContext {
    pub api_key: String,
    pub api_secret: String,
    pub upload_endpoint: String,
    pub delete_endpoint: String,
    pub upload_preset: String,
}

#[derive(Clone)]
pub struct PaymentContext {
    pub public_key: String,
    pub secret_key: String,
}

#[derive(Clone)]
pub struct Context {
    pub app: AppContext,
    pub db_conn: database::DatabaseConnection,
    pub storage: StorageContext,
    pub payment: PaymentContext,
}

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
pub struct PaymentConfig {
    pub public_key: String,
    pub secret_key: String,
}

#[derive(Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub app: AppConfig,
    pub storage: StorageConfig,
    pub payment: PaymentConfig,
}

impl Default for Config {
    fn default() -> Self {
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
        let payment_public_key =
            env::var("PAYSTACK_PUBLIC_KEY").expect("PAYSTACK_PUBLIC_KEY not set");
        let payment_secret_key =
            env::var("PAYMENT_SECRET_KEY").expect("PAYMENT_SECRET_KEY not set");

        return Self {
            database: DatabaseConfig { url: database_url },
            app: AppConfig { host, port, url },
            storage: StorageConfig {
                api_key: storage_api_key,
                api_secret: storage_api_secret,
                upload_endpoint: storage_upload_endpoint,
                delete_endpoint: storage_delete_endpoint,
                upload_preset: storage_upload_preset,
            },
            payment: PaymentConfig {
                public_key: payment_public_key,
                secret_key: payment_secret_key,
            },
        };
    }
}

#[async_trait]
pub trait ToContext {
    async fn to_context(self) -> Context;
}

#[async_trait]
impl ToContext for Config {
    async fn to_context(self) -> Context {
        let db_conn = database::connect(self.database.url.as_str()).await;
        database::migrate(db_conn.clone()).await;

        Context {
            app: AppContext {
                host: self.app.host,
                port: self.app.port,
                url: self.app.url,
            },
            db_conn: db_conn.clone(),
            storage: StorageContext {
                api_key: self.storage.api_key,
                api_secret: self.storage.api_secret,
                upload_endpoint: self.storage.upload_endpoint,
                delete_endpoint: self.storage.delete_endpoint,
                upload_preset: self.storage.upload_preset,
            },
            payment: PaymentContext {
                public_key: self.payment.public_key,
                secret_key: self.payment.secret_key,
            },
        }
    }
}
