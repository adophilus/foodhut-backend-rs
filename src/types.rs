pub use crate::utils::database;
use apalis::prelude::MemoryStorage;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use core::time::Duration;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, Mutex};

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
pub struct MailContext {
    pub access_token: Arc<Mutex<String>>,
    pub refresh_token: String,
    pub refresh_endpoint: String,
    pub sender_name: String,
    pub sender_email: String,
}

#[derive(Clone)]
pub struct GoogleContext {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Clone)]
pub struct Context {
    pub app: AppContext,
    pub db_conn: database::DatabaseConnection,
    pub storage: StorageContext,
    pub payment: PaymentContext,
    pub mail: MailContext,
    pub google: GoogleContext,
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
pub struct MailConfig {
    pub access_token: String,
    pub refresh_token: String,
    pub refresh_endpoint: String,
    pub sender_name: String,
    pub sender_email: String,
}

#[derive(Clone)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub app: AppConfig,
    pub storage: StorageConfig,
    pub payment: PaymentConfig,
    pub mail: MailConfig,
    pub google: GoogleConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Job(DateTime<Utc>);

impl apalis::prelude::Job for Job {
    const NAME: &'static str = "apalis::Job";
}

impl From<DateTime<Utc>> for Job {
    fn from(t: DateTime<Utc>) -> Self {
        Self(t)
    }
}

#[derive(Clone)]
pub struct JobStorage {
    memory_storage: apalis::prelude::MemoryStorage<Job>,
    storage: Vec<Job>,
}

impl JobStorage {
    pub fn new() -> Self {
        Self {
            memory_storage: MemoryStorage::<Job>::new(),
            storage: vec![],
        }
    }
}

impl apalis::prelude::Backend<apalis::prelude::Request<Job>> for JobStorage {
    type Stream = apalis::prelude::BackendStream<
        apalis::prelude::RequestStream<apalis::prelude::Request<Job>>,
    >;

    type Layer = tower::ServiceBuilder<tower::layer::util::Identity>;

    fn common_layer(&self, worker: apalis::prelude::WorkerId) -> Self::Layer {
        self.memory_storage.common_layer(worker)
    }

    fn poll(self, worker: apalis::prelude::WorkerId) -> apalis::prelude::Poller<Self::Stream> {
        self.memory_storage.poll(worker)
    }
}

impl apalis::prelude::Storage for JobStorage {
    type Job = Job;

    type Error = apalis::prelude::Error;

    type Identifier = usize;

    async fn push(&mut self, job: Self::Job) -> Result<Self::Identifier, Self::Error> {
        tracing::debug!("Job pushed to storage");
        self.storage.push(job);
        Ok(self.storage.len())
    }

    async fn schedule(&mut self, job: Self::Job, on: i64) -> Result<Self::Identifier, Self::Error> {
        tracing::debug!("Job pushed into the schedule set");
        todo!()
    }

    async fn len(&self) -> Result<i64, Self::Error> {
        tracing::debug!("Returning number of pending jobs");
        Ok(self.storage.len() as i64)
    }

    async fn fetch_by_id(
        &self,
        job_id: &Self::Identifier,
    ) -> Result<Option<apalis::prelude::Request<Self::Job>>, Self::Error> {
        tracing::debug!("Fetching job by id: {}", job_id);
        // let job = self.jobs.get(job_id);
        todo!()
    }

    async fn update(&self, job: apalis::prelude::Request<Self::Job>) -> Result<(), Self::Error> {
        tracing::debug!("Updating job details");
        todo!()
    }

    async fn reschedule(
        &mut self,
        job: apalis::prelude::Request<Self::Job>,
        wait: Duration,
    ) -> Result<(), Self::Error> {
        tracing::debug!("Rescheduling job");
        todo!()
    }

    async fn is_empty(&self) -> Result<bool, Self::Error> {
        tracing::debug!("Determining whether there's still any job in the storage");
        todo!()
    }

    async fn vacuum(&self) -> Result<usize, Self::Error> {
        tracing::debug!("Vacuuming queue");
        todo!()
    }
}

pub trait SchedulableJob: Send + Sync + Clone {
    fn schedule(&self) -> apalis::cron::Schedule;
    fn run(&self) -> impl std::future::Future<Output = Result<(), apalis::prelude::Error>> + Send;
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
            env::var("PAYSTACK_SECRET_KEY").expect("PAYSTACK_SECRET_KEY not set");
        let mail_access_token = env::var("MAIL_ACCESS_TOKEN").expect("MAIL_ACCESS_TOKEN not set");
        let mail_refresh_token =
            env::var("MAIL_REFRESH_TOKEN").expect("MAIL_REFRESH_TOKEN not set");
        let mail_refresh_endpoint =
            env::var("MAIL_REFRESH_ENDPOINT").expect("MAIL_REFRESH_ENDPOINT not set");
        let mail_sender_name = env::var("MAIL_SENDER_NAME").expect("MAIL_SENDER_NAME not set");
        let mail_sender_email = env::var("MAIL_SENDER_EMAIL").expect("MAIL_SENDER_EMAIL not set");
        let google_client_id = env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID not set");
        let google_client_secret =
            env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET not set");

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
            mail: MailConfig {
                access_token: mail_access_token,
                refresh_token: mail_refresh_token,
                refresh_endpoint: mail_refresh_endpoint,
                sender_name: mail_sender_name,
                sender_email: mail_sender_email,
            },
            google: GoogleConfig {
                client_id: google_client_id,
                client_secret: google_client_secret,
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
            mail: MailContext {
                access_token: Arc::new(Mutex::new(self.mail.access_token)),
                refresh_token: self.mail.refresh_token,
                refresh_endpoint: self.mail.refresh_endpoint,
                sender_name: self.mail.sender_name,
                sender_email: self.mail.sender_email,
            },
            google: GoogleContext {
                client_id: self.google.client_id,
                client_secret: self.google.client_secret,
            },
        }
    }
}
