use migration_main::MigratorTrait;
use serde_json::json;
use uuid::Uuid;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    db::{
        main::{self, ColumnTrait, EntityTrait, QueryFilter},
        tenant,
    },
    utils::api_response::ApiResponse,
};

pub async fn migrate_tenants(
    main_db: &main::DatabaseConnection,
) -> Result<Arc<RwLock<HashMap<Uuid, tenant::DatabaseConnection>>>, ApiResponse> {
    let tenant_dbs = Arc::new(RwLock::new(HashMap::new()));

    main::Migrator::up(main_db, None).await.map_err(|err| {
        log::error!("Failed to migrate main DB: {}", err);
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

        let tenant_db = tenant::Database::connect(&db_url).await.map_err(|err| {
            log::error!("Failed to connect tenant {}: {}", tenant_id, err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

        tenant::Migrator::up(&tenant_db, None)
            .await
            .map_err(|err| {
                log::error!("Tenant {} migration failed: {}", tenant_id, err);
                ApiResponse::new(500, json!({ "message": err.to_string() }))
            })?;

        tenant_dbs.write().unwrap().insert(tenant_id, tenant_db);
    }

    Ok(tenant_dbs)
}
