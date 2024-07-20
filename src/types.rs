pub use crate::database::DatabaseConnection;

#[derive(Clone)]
pub struct StorageContext {
    pub api_key: String,
    pub api_secret: String,
    pub upload_endpoint: String,
    pub delete_endpoint: String,
    pub upload_preset: String,
}

#[derive(Clone)]
pub struct AppContext {
    pub host: String,
    pub port: u32,
    pub url: String,
}

#[derive(Clone)]
pub struct Context {
    pub app: AppContext,
    pub db_conn: DatabaseConnection,
    pub storage: StorageContext,
}
