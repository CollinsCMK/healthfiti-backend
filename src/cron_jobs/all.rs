use sea_orm::DatabaseConnection;
use serde_json::json;
use tokio_cron_scheduler::{JobScheduler, Job};

use crate::utils::api_response::ApiResponse;

pub async fn init_cron_jobs(db: &DatabaseConnection) -> Result<(), ApiResponse> {
    let sched = JobScheduler::new()
        .await
        .map_err(|err| {
            log::error!("Failed to create scheduler: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to create scheduler" }))
        })?;

    // let job = Job::new_async("0 0 0 * * *", move |_uuid, _l| {
    //     let db = db.clone();
    //     Box::pin(async move {
    //         if let Err(err) = process_trial_expiry(&db).await {
    //             log::error!("Trial expiry error: {}", err);
    //         }
    //     })
    // })?;

    // sched.add(job).await?;
    // sched.start().await?;

    Ok(())
}
