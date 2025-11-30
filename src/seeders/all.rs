use serde_json::json;
use std::{future::Future, pin::Pin};

use crate::{
    db::{main, tenant},
    seeders::main::permissions::seed_permissions,
    utils::api_response::ApiResponse,
};

pub async fn seed_main_all(
    db: &main::migrations::sea_orm::DatabaseConnection,
) -> Result<ApiResponse, ApiResponse> {
    // Define a type alias for clarity with lifetime
    type SeederFn<'a> =
        fn(
            &'a main::migrations::sea_orm::DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = Result<ApiResponse, ApiResponse>> + Send + 'a>>;

    // Use explicit lifetime annotation here
    let seeders: Vec<SeederFn<'_>> = vec![|_db| Box::pin(seed_permissions())];

    for seeder in seeders {
        let res = seeder(db).await?;
        if res.status_code != 200 {
            return Err(res);
        }
    }

    Ok(ApiResponse::new(
        200,
        json!({ "message": "All main seeders ran successfully".to_string() }),
    ))
}

pub async fn seed_tenant_all(
    db: &tenant::migrations::sea_orm::DatabaseConnection,
) -> Result<ApiResponse, ApiResponse> {
    // Define a type alias for clarity with lifetime
    type SeederFn<'a> =
        fn(
            &'a tenant::migrations::sea_orm::DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = Result<ApiResponse, ApiResponse>> + Send + 'a>>;

    // Use explicit lifetime annotation here
    let seeders: Vec<SeederFn<'_>> = vec![
        // |db| Box::pin(seed_permissions(db)),
    ];

    for seeder in seeders {
        let res = seeder(db).await?;
        if res.status_code != 200 {
            return Err(res);
        }
    }

    Ok(ApiResponse::new(
        200,
        json!({ "message": "All tenant seeders ran successfully".to_string() }),
    ))
}
