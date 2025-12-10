use actix_web::web::{self};

use crate::{handlers::tenant::subscription_plans, middlewares::permissions::Permission};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/subscription-plans")
            .service(
                web::resource("")
                    .wrap(Permission::new("view_subscription_plans".to_string()))
                    .route(web::get().to(subscription_plans::index)),
            )
            .service(
                web::resource("/show/{pid}")
                    .wrap(Permission::new("view_subscription_plan".to_string()))
                    .route(web::get().to(subscription_plans::show)),
            ),
    );
}
