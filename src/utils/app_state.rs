use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use actix_web::web;
use aws_sdk_s3::Client;
use redis::Client as RedisClient;
use uuid::Uuid;

use crate::{
    db::{main, tenant},
    utils::message_queue::MessageQueue,
};

pub struct AppState {
    pub main_db: main::migrations::sea_orm::DatabaseConnection,
    pub tenant_dbs: Arc<RwLock<HashMap<Uuid, tenant::migrations::sea_orm::DatabaseConnection>>>,
    pub s3_client: Client,
    pub bucket: String,
    pub message_queue: web::Data<MessageQueue>,
    pub redis: RedisClient,
}

impl AppState {
    pub fn tenant_db(
        &self,
        tenant_id: Uuid,
    ) -> Option<tenant::migrations::sea_orm::DatabaseConnection> {
        self.tenant_dbs.read().unwrap().get(&tenant_id).cloned()
    }
}
