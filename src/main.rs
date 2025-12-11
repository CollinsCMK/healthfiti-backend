use std::{error::Error, fmt::Display, time::Duration};

use actix_cors::Cors;
use actix_extensible_rate_limit::{
    RateLimiter,
    backend::{SimpleInputFunctionBuilder, memory::InMemoryBackend},
};
use actix_web::{App, HttpServer, middleware::Logger, web};
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_s3::{Client, Config};

use crate::{
    cron_jobs::all::init_cron_jobs,
    db::main,
    utils::{app_state::AppState, message_queue::init_message_queue, migrate::migrate_tenants},
};

mod cron_jobs;
mod db;
mod emails;
mod handlers;
mod middlewares;
mod routes;
mod seeders;
mod utils;

#[derive(Debug)]
struct MainError {
    message: String,
}

impl Display for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl Error for MainError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> Result<(), MainError> {
    if std::env::var_os("RUST_LOG").is_none() {
        unsafe { std::env::set_var("RUST_LOG", "actix_web=info") }
    }

    dotenv::dotenv().ok();
    env_logger::init();

    let minio_endpoint = (utils::constants::MINIO_ENDPOINT).clone();
    let minio_access_key = (utils::constants::MINIO_ACCESS_KEY).clone();
    let minio_secret_key = (utils::constants::MINIO_SECRET_KEY).clone();
    let minio_region = (utils::constants::MINIO_REGION).clone();
    let minio_bucket = (utils::constants::MINIO_BUCKET).clone();
    let credentials = Credentials::new(minio_access_key, minio_secret_key, None, None, "static");

    let config = Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(Region::new(minio_region))
        .endpoint_url(minio_endpoint)
        .force_path_style(true)
        .build();

    let client = Client::from_conf(config);

    match client.head_bucket().bucket(&minio_bucket).send().await {
        Ok(_) => {}
        Err(_) => {
            client
                .create_bucket()
                .bucket(&minio_bucket)
                .send()
                .await
                .map_err(|err| MainError {
                    message: format!("Failed to create bucket: {}", err),
                })?;
        }
    }

    let port = (utils::constants::PORT).clone();
    let address = (utils::constants::ADDRESS).clone();
    let database_url = (utils::constants::DATABASE_URL).clone();
    let max_file_size = (utils::constants::MAX_FILE_SIZE).clone() as usize;
    let redis_url = (utils::constants::REDIS_URL).clone();

    let main_db: main::migrations::sea_orm::DatabaseConnection =
        main::migrations::sea_orm::Database::connect(database_url)
            .await
            .map_err(|err| MainError {
                message: err.to_string(),
            })?;

    let tenant_dbs = migrate_tenants(&main_db).await.map_err(|err| MainError {
        message: err.to_string(),
    })?;

    let allowed_origins = (utils::constants::ALLOWED_ORIGINS).clone();

    let redis_client = redis::Client::open(redis_url.clone()).map_err(|err| MainError {
        message: format!("Redis client error: {}", err),
    })?;

    let message_queue = init_message_queue(&redis_url);

    init_cron_jobs(&main_db).await.map_err(|err| MainError {
        message: err.to_string(),
    })?;

    let backend = InMemoryBackend::builder().build();

    HttpServer::new(move || {
        let input = SimpleInputFunctionBuilder::new(Duration::from_secs(60), 100)
            .real_ip_key()
            .build();
        let middleware = RateLimiter::builder(backend.clone(), input)
            .add_headers()
            .build();

        let mut cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);

        for origin in &allowed_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .app_data(web::Data::new(AppState {
                main_db: main_db.clone(),
                tenant_dbs: tenant_dbs.clone(),
                s3_client: client.clone(),
                bucket: minio_bucket.clone(),
                message_queue: message_queue.clone(),
                redis: redis_client.clone(),
            }))
            .app_data(web::PayloadConfig::new(max_file_size))
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(middleware)
            .configure(routes::api::config)
    })
    .bind((address, port))
    .map_err(|err| MainError {
        message: err.to_string(),
    })?
    .run()
    .await
    .map_err(|err| MainError {
        message: err.to_string(),
    })
}
