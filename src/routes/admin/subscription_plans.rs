use actix_web::web::{self};

use crate::{
    handlers::admin::subscription_plans,
    middlewares::permissions::Permission,
};

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
            )
            .service(
                web::resource("/create")
                    .wrap(Permission::new("create_subscription_plan".to_string()))
                    .route(web::post().to(subscription_plans::create)),
            )
            .service(
                web::resource("/edit/{pid}")
                    .wrap(Permission::new("update_subscription_plan".to_string()))
                    .route(web::put().to(subscription_plans::edit)),
            )
            .service(
                web::resource("/status/{pid}")
                    .wrap(Permission::new("activate_or_deactivate_subscription_plan".to_string()))
                    .route(web::patch().to(subscription_plans::set_active_status)),
            )
            .service(
                web::resource("/delete/{pid}")
                    .wrap(Permission::new("soft_delete_subscription_plan".to_string()))
                    .route(web::delete().to(subscription_plans::destroy)),
            )
            .service(
                web::resource("/restore/{pid}")
                    .wrap(Permission::new("restore_subscription_plan".to_string()))
                    .route(web::put().to(subscription_plans::restore)),
            )
            .service(
                web::resource("/permanent/{pid}")
                    .wrap(Permission::new("delete_subscription_plan".to_string()))
                    .route(web::delete().to(subscription_plans::delete_permanently)),
            ),
    );
}
