use actix_web::web::{self};

use crate::{handlers::admin::payments, middlewares::permissions::Permission};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/payments")
            .service(
                web::resource("")
                    .wrap(Permission::new("view_all_payment_transactions".to_string()))
                    .route(web::get().to(payments::index)),
            )
            .service(
                web::resource("/show/{pid}")
                    .wrap(Permission::new("view_payment_transaction".to_string()))
                    .route(web::get().to(payments::show)),
            )
            .service(
                web::resource("/show/tenant/{pid}")
                    .wrap(Permission::new("view_tenant_payment_transaction".to_string()))
                    .route(web::get().to(payments::show_by_tenant)),
            )
            .service(
                web::resource("/status/{pid}")
                    .wrap(Permission::new("activate_or_deactivate_payment_transaction".to_string()))
                    .route(web::post().to(payments::set_active_status)),
            )
    );
}
