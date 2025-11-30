use migration_main::MigratorTrait;
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use uuid::Uuid;

use crate::{
    db::{
        main::{
            self,
            migrations::sea_orm::{ColumnTrait, EntityTrait, QueryFilter},
        },
        tenant,
    },
    seeders::all::{seed_main_all, seed_tenant_all},
    utils::api_response::ApiResponse,
};

pub async fn migrate_tenants(
    main_db: &main::migrations::sea_orm::DatabaseConnection,
) -> Result<Arc<RwLock<HashMap<Uuid, tenant::migrations::sea_orm::DatabaseConnection>>>, ApiResponse>
{
    let tenant_dbs = Arc::new(RwLock::new(HashMap::new()));

    main::migrations::Migrator::up(main_db, None)
        .await
        .map_err(|err| {
            log::error!("Failed to migrate main DB: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    seed_main_all(&main_db).await.map_err(|err| {
        log::error!("Failed to seed main db data: {}", err);
        ApiResponse::new(500, json!({ "message": err.to_string() }))
    })?;

    let tenants = entity_main::tenants::Entity::find()
        .filter(entity_main::tenants::Column::DeletedAt.is_null())
        .all(main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch tenants: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    for tenant in tenants {
        let tenant_id = tenant.sso_tenant_id;
        let db_url = tenant.db_url;

        let tenant_db = tenant::migrations::sea_orm::Database::connect(&db_url)
            .await
            .map_err(|err| {
                log::error!("Failed to connect tenant {}: {}", tenant_id, err);
                ApiResponse::new(500, json!({ "message": err.to_string() }))
            })?;

        tenant::migrations::Migrator::up(&tenant_db, None)
            .await
            .map_err(|err| {
                log::error!("Tenant {} migration failed: {}", tenant_id, err);
                ApiResponse::new(500, json!({ "message": err.to_string() }))
            })?;

        seed_tenant_all(&tenant_db).await.map_err(|err| {
            log::error!("Failed to seed tenant db data {}: {}", tenant_id, err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

        tenant_dbs.write().unwrap().insert(tenant_id, tenant_db);
    }

    Ok(tenant_dbs)
}
