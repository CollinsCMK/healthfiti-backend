use serde_json::json;
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        migrations::sea_orm::{ColumnTrait, QueryFilter},
    },
    utils::{api_response::ApiResponse, app_state::AppState},
};

pub async fn get_insurance_id(app_state: &AppState, id: Uuid) -> Result<i32, ApiResponse> {
    let id = main::entities::insurance_providers::Entity::find_by_pid(id)
        .filter(main::entities::insurance_providers::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch insurance provider: {:?}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to fetch insurance provider. Please try again.",
                    "error": err.to_string()
                }),
            )
        })?
        .ok_or_else(|| {
            log::error!("Insurance provider not found");

            ApiResponse::new(
                404,
                json!({
                    "message": "Insurance provider not found"
                }),
            )
        })?
        .id;

    Ok(id)
}
