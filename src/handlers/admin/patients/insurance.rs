use actix_web::{HttpRequest, web};
use serde_json::json;

use crate::{
    db::main::{
        self,
        migrations::sea_orm::{Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder},
    },
    utils::{api_response::ApiResponse, app_state::AppState, pagination::PaginationParams},
};

pub async fn index(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
) -> Result<ApiResponse, ApiResponse> {
    let fetch_all = query.all.unwrap_or(false);

    let mut stmt = main::entities::patients::Entity::find()
        .find_also_related(main::entities::patient_insurance::Entity);

    if let Some(term) = &query.search {
        use main::migrations::{Expr, extension::postgres::PgExpr};

        let like = format!("%{}%", term);

        stmt = stmt.filter(
            Condition::any()
                .add(
                    Expr::col((
                        main::entities::patient_insurance::Entity,
                        main::entities::patient_insurance::Column::Provider,
                    ))
                    .ilike(like.clone()),
                )
                .add(
                    Expr::col((
                        main::entities::patient_insurance::Entity,
                        main::entities::patient_insurance::Column::PolicyNumber,
                    ))
                    .ilike(like.clone()),
                )
                .add(
                    Expr::col((
                        main::entities::patient_insurance::Entity,
                        main::entities::patient_insurance::Column::GroupNumber,
                    ))
                    .ilike(like.clone()),
                ),
        );
    }

    if fetch_all {
        let model = stmt
            .order_by_asc(main::entities::patient_insurance::Column::CreatedAt)
            .order_by_asc(main::entities::patient_insurance::Column::Id)
            .all(&app_state.main_db)
            .await
            .map_err(|err| ApiResponse::new(500, json!({"message": err.to_string()})))?
            .into_iter()
            .map(|(_, ins)| {
                if let Some(ins) = ins {
                    json!({
                        "pid": ins.pid,
                        "provider": ins.provider,
                        "policy_number": ins.policy_number,
                        "group_number": ins.group_number,
                        "plan_type": ins.plan_type,
                        "coverage_start_date": ins.coverage_start_date,
                        "coverage_end_date": ins.coverage_end_date,
                        "is_primary": ins.is_primary,
                    })
                } else {
                    json!(null)
                }
            })
            .collect::<Vec<_>>();

        return Ok(ApiResponse::new(
            200,
            json!({
                "insurances": model,
                "success": "Patient insurance fetched successfully"
            }),
        ));
    }

    let page = query.page.unwrap_or(1).min(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let paginator = stmt.paginate(&app_state.main_db, limit);

    let total_items = paginator
        .num_items()
        .await
        .map_err(|err| ApiResponse::new(500, json!({"message": err.to_string()})))?;

    let total_pages = (total_items as f64 / limit as f64).ceil() as u64;
    let has_prev = page > 1;
    let has_next = page < total_pages;

    let model = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .map_err(|err| ApiResponse::new(500, json!({"message": err.to_string()})))?
        .into_iter()
        .map(|(patient, ins)| {
            if let Some(ins) = ins {
                json!({
                    "pid": ins.pid,
                    "provider": ins.provider,
                    "policy_number": ins.policy_number,
                    "group_number": ins.group_number,
                    "plan_type": ins.plan_type,
                    "coverage_start_date": ins.coverage_start_date,
                    "coverage_end_date": ins.coverage_end_date,
                    "is_primary": ins.is_primary,
                    "patient": {
                        "pid": patient.pid,
                        "name": format!("{:?} {:?}", patient.first_name, patient.last_name),
                    },
                    "created_at": ins.created_at,
                })
            } else {
                json!(null)
            }
        })
        .collect::<Vec<_>>();

    Ok(ApiResponse::new(
        200,
        json!({
            "insurances": model,
            "page": page,
            "total_pages": total_pages,
            "total_items": total_items,
            "has_prev": has_prev,
            "has_next": has_next,
            "message": "Patient insurance fetched successfully",
        }),
    ))
}
