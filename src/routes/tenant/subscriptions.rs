use actix_web::web::{self};

use crate::{handlers::tenant::subscriptions, middlewares::permissions::Permission};

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
                web::resource("/trial/{pid}")
                    .wrap(Permission::new("trial_subscription".to_string()))
                    .route(web::post().to(subscriptions::trial)),
            )
            .service(
                web::resource("/cancel/{pid}")
                    .wrap(Permission::new("cancel_subscription".to_string()))
                    .route(web::post().to(subscriptions::cancel)),
            ),
    );
}
