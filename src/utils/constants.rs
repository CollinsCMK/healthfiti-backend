use std::env;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref APP_NAME: String = set_app_name();
    pub static ref APP_DESCRIPTION: String = set_app_description();
    pub static ref ADDRESS: String = set_address();
    pub static ref PORT: u16 = set_port();
    pub static ref APP_URL: String = set_app_url();
    pub static ref DATABASE_URL: String = set_database_url();
    pub static ref MAX_FILE_SIZE: u64 = set_max_file_size();
    // pub static ref MAIL_PORT: u16 = mail_port();
    pub static ref MAIL_HOST: String = mail_host();
    pub static ref MAIL_USERNAME: String = mail_username();
    pub static ref MAIL_PASSWORD: String = mail_password();
    pub static ref MAIL_FROM_ADDRESS: String = mail_from_address();
    pub static ref VASPRO_API_KEY: String = vaspro_api_key();
    pub static ref VASPROT_SHORTCODE: String = vaspro_shortcode();
    pub static ref REDIS_URL: String = redis_url();
    pub static ref SECRET: String = set_secret();
    pub static ref MINIO_ENDPOINT: String = set_minio_endpoint();
    pub static ref MINIO_ACCESS_KEY: String = set_minio_access_key();
    pub static ref MINIO_SECRET_KEY: String = set_minio_secret_key();
    pub static ref MINIO_REGION: String = set_minio_region();
    pub static ref MINIO_BUCKET: String = set_minio_bucket();
    pub static ref ALLOWED_ORIGINS: Vec<String> = allowed_origins();
    pub static ref SSO_BASE_URL: String = sso_base_url();
    pub static ref SSO_CLIENT_ID: String = sso_client_id();
    pub static ref SSO_CLIENT_SECRET: String = sso_client_secret();
    pub static ref MPESA_BASE_URL: String = mpesa_base_url();
    pub static ref MPESA_CONSUMER_KEY: String = mpesa_consumer_key();
    pub static ref MPESA_CONSUMER_SECRET: String = mpesa_consumer_secret();
    pub static ref MPESA_SHORTCODE: String = mpesa_shortcode();
}

fn set_app_name() -> String {
    dotenv::dotenv().ok();
    env::var("APP_NAME").expect("Environment variable 'APP_NAME' is required but not set.")
}

fn set_app_description() -> String {
    dotenv::dotenv().ok();
    env::var("APP_DESCRIPTION")
        .expect("Environment variable 'APP_DESCRIPTION' is required but not set.")
}

fn set_address() -> String {
    dotenv::dotenv().ok();
    env::var("ADDRESS").expect("Environment variable 'ADDRESS' is required but not set.")
}

fn set_port() -> u16 {
    dotenv::dotenv().ok();
    env::var("PORT")
        .expect("Environment variable 'PORT' is required but not set.")
        .parse::<u16>()
        .expect("Failed to parse 'PORT' as a valid u16 value.")
}

fn set_app_url() -> String {
    dotenv::dotenv().ok();
    env::var("APP_URL").expect("Environment variable 'APP_URL' is required but not set.")
}

fn set_database_url() -> String {
    dotenv::dotenv().ok();
    env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' is required but not set.")
}

fn set_max_file_size() -> u64 {
    dotenv::dotenv().ok();
    env::var("MAX_FILE_SIZE")
        .unwrap_or("10485760".to_owned())
        .parse::<u64>()
        .expect("Can't parse that file size")
}

// fn mail_port() -> u16 {
//     dotenv::dotenv().ok();
//     env::var("MAIL_PORT")
//         .expect("Environment variable 'MAIL_PORT' is required but not set.")
//         .parse::<u16>()
//         .expect("Failed to parse 'MAIL_PORT' as a valid u16 value.")
// }

fn mail_host() -> String {
    dotenv::dotenv().ok();
    env::var("MAIL_HOST").expect("Environment variable 'MAIL_HOST' is required but not set.")
}

fn mail_username() -> String {
    dotenv::dotenv().ok();
    env::var("MAIL_USERNAME")
        .expect("Environment variable 'MAIL_USERNAME' is required but not set.")
}

fn mail_password() -> String {
    dotenv::dotenv().ok();
    env::var("MAIL_PASSWORD")
        .expect("Environment variable 'MAIL_PASSWORD' is required but not set.")
}

fn mail_from_address() -> String {
    dotenv::dotenv().ok();
    env::var("MAIL_FROM_ADDRESS")
        .expect("Environment variable 'MAIL_FROM_ADDRESS' is required but not set.")
}

fn vaspro_api_key() -> String {
    dotenv::dotenv().ok();
    env::var("VASPRO_API_KEY")
        .expect("Environment variable 'VASPRO_API_KEY' is required but not set.")
}

fn vaspro_shortcode() -> String {
    dotenv::dotenv().ok();
    env::var("VASPRO_SHORTCODE")
        .expect("Environment variable 'VASPRO_SHORTCODE' is required but not set.")
}

fn redis_url() -> String {
    dotenv::dotenv().ok();
    env::var("REDIS_URL").expect("Environment variable 'REDIS_URL' is required but not set.")
}

fn set_secret() -> String {
    dotenv::dotenv().ok();
    env::var("SECRET").expect("Environment variable 'SECRET' is required but not set.")
}

fn set_minio_endpoint() -> String {
    dotenv::dotenv().ok();
    env::var("MINIO_ENDPOINT")
        .expect("Environment variable 'MINIO_ENDPOINT' is required but not set.")
}

fn set_minio_access_key() -> String {
    dotenv::dotenv().ok();
    env::var("MINIO_ACCESS_KEY")
        .expect("Environment variable 'MINIO_ACCESS_KEY' is required but not set.")
}

fn set_minio_secret_key() -> String {
    dotenv::dotenv().ok();
    env::var("MINIO_SECRET_KEY")
        .expect("Environment variable 'MINIO_SECRET_KEY' is required but not set.")
}

fn set_minio_region() -> String {
    dotenv::dotenv().ok();
    env::var("MINIO_REGION").expect("Environment variable 'MINIO_REGION' is required but not set.")
}

fn set_minio_bucket() -> String {
    dotenv::dotenv().ok();
    env::var("MINIO_BUCKET").expect("Environment variable 'MINIO_BUCKET' is required but not set.")
}

fn allowed_origins() -> Vec<String> {
    dotenv::dotenv().ok();
    env::var("ALLOWED_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>()
}

fn sso_base_url() -> String {
    dotenv::dotenv().ok();
    env::var("SSO_BASE_URL").expect("Environment variable 'SSO_BASE_URL' is required but not set.")
}

fn sso_client_id() -> String {
    dotenv::dotenv().ok();
    env::var("SSO_CLIENT_ID")
        .expect("Environment variable 'SSO_CLIENT_ID' is required but not set.")
}

fn sso_client_secret() -> String {
    dotenv::dotenv().ok();
    env::var("SSO_CLIENT_SECRET")
        .expect("Environment variable 'SSO_CLIENT_SECRET' is required but not set.")
}

fn mpesa_base_url() -> String {
    dotenv::dotenv().ok();
    env::var("MPESA_BASE_URL").expect("Environment variable 'MPESA_BASE_URL' is required but not set.")
}

fn mpesa_consumer_key() -> String {
    dotenv::dotenv().ok();
    env::var("MPESA_CONSUMER_KEY").expect("Environment variable 'MPESA_CONSUMER_KEY' is required but not set.")
}

fn mpesa_consumer_secret() -> String {
    dotenv::dotenv().ok();
    env::var("MPESA_CONSUMER_SECRET").expect("Environment variable 'MPESA_CONSUMER_SECRET' is required but not set.")
}

fn mpesa_shortcode() -> String {
    dotenv::dotenv().ok();
    env::var("MPESA_SHORTCODE").expect("Environment variable 'MPESA_SHORTCODE' is required but not set.")
}
