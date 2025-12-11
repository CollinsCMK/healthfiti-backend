use serde_json::Value;
use serde_json::json;

use crate::db::main::{
    self,
    migrations::sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter},
};
use crate::utils::{api_response::ApiResponse, app_state::AppState, constants};

pub async fn email_configs(
    app_state: &AppState,
    domain: Option<String>,
) -> Result<(String, String, String, String, String, String, String), ApiResponse> {
    let tenant_branding = if let Some(d) = domain.clone() {
        use main::migrations::{Expr, extension::postgres::PgExpr};

        let tenant = main::entities::tenants::Entity::find()
            .filter(main::entities::tenants::Column::DeletedAt.is_null())
            .filter(
                Condition::any()
                    .add(
                        Expr::col(main::entities::tenants::Column::Settings)
                            .contains(format!(r#"{{"admin_subdomain":"{}"}}"#, d)),
                    )
                    .add(
                        Expr::col(main::entities::tenants::Column::Settings)
                            .contains(format!(r#"{{"admin_domain":"{}"}}"#, d)),
                    )
                    .add(
                        Expr::col(main::entities::tenants::Column::Settings)
                            .contains(format!(r#"{{"patient_subdomain":"{}"}}"#, d)),
                    )
                    .add(
                        Expr::col(main::entities::tenants::Column::Settings)
                            .contains(format!(r#"{{"patient_domain":"{}"}}"#, d)),
                    ),
            )
            .one(&app_state.main_db)
            .await
            .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

        tenant.and_then(|t| t.settings.and_then(|s| s.get("branding").cloned()))
    } else {
        None
    };

    let branding = tenant_branding.unwrap_or_else(|| json!({}));

    let logo_url = branding
        .get("logo_url")
        .and_then(Value::as_str)
        .unwrap_or(&constants::APP_LOGO_URL)
        .to_string();

    let privacy_url = branding
        .get("privacy_url")
        .and_then(Value::as_str)
        .unwrap_or(&constants::APP_PRIVACY_URL)
        .to_string();

    let app_name = branding
        .get("app_name")
        .and_then(Value::as_str)
        .unwrap_or(&constants::APP_NAME)
        .to_string();

    let primary_color = branding
        .get("primary_color")
        .and_then(Value::as_str)
        .unwrap_or(&constants::APP_PRIMARY_COLOR)
        .to_string();

    let accent_color = branding
        .get("accent_color")
        .and_then(Value::as_str)
        .unwrap_or(&constants::APP_ACCENT_COLOR)
        .to_string();

    let text_color = branding
        .get("text_color")
        .and_then(Value::as_str)
        .unwrap_or(&constants::APP_TEXT_COLOR)
        .to_string();

    let footer_text_color = branding
        .get("footer_text_color")
        .and_then(Value::as_str)
        .unwrap_or(&constants::APP_FOOTER_TEXT_COLOR)
        .to_string();

    Ok((
        logo_url,
        privacy_url,
        app_name,
        primary_color,
        accent_color,
        text_color,
        footer_text_color,
    ))
}
