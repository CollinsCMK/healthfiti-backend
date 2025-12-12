use actix_web::web::{self};

use crate::{handlers::admin::subscriptions, middlewares::permissions::Permission};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/subscriptions")
            .service(
                web::resource("")
                    .wrap(Permission::new("view_all_subscriptions".to_string()))
                    .route(web::get().to(subscriptions::index)),
            )
            .service(
                web::resource("/show/{pid}")
                    .wrap(Permission::new("view_subscription".to_string()))
                    .route(web::get().to(subscriptions::show)),
            )
            .service(
                web::resource("/status/{pid}")
                    .wrap(Permission::new(
                        "activate_or_deactivate_subscription".to_string(),
                    ))
                    .route(web::post().to(subscriptions::set_active_status)),
            ),
    );
}
