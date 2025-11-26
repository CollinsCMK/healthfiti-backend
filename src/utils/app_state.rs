use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use aws_sdk_s3::Client;
use uuid::Uuid;

use crate::db::{main, tenant};

pub struct AppState {
    pub main_db: main::DatabaseConnection,
    pub tenant_dbs: Arc<RwLock<HashMap<Uuid, tenant::DatabaseConnection>>>,
    pub s3_client: Client,
    pub bucket: String,
}

impl AppState {
    pub fn tenant_db(&self, tenant_id: Uuid) -> Option<tenant::DatabaseConnection> {
        self.tenant_dbs.read().unwrap().get(&tenant_id).cloned()
    }
}
