use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use aws_sdk_s3::Client;
use migration_main::sea_orm::DatabaseConnection as MainDatabaseConnection;
use migration_tenant::sea_orm::DatabaseConnection as TenantDatabaseConnection;

pub struct AppState {
    pub main_db: MainDatabaseConnection,
    pub tenant_dbs: Arc<RwLock<HashMap<String, TenantDatabaseConnection>>>,
    pub s3_client: Client,
    pub bucket: String,
}
