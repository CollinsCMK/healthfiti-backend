use actix_web::web::{self};

use crate::{
    handlers::tenant::payments,
    middlewares::{jwt_auth::JwtAuth, permissions::Permission},
    utils,
};

pub fn config(config: &mut web::ServiceConfig) {
    let secret = (utils::constants::SECRET).clone();

    config.service(
        web::scope("/payments")
            .service(
                web::resource("")
                    .wrap(JwtAuth)
                    .wrap(Permission::new("view_all_payment_transactions".to_string()))
                    .route(web::get().to(payments::index)),
            )
            .service(
                web::resource("/show/{pid}")
                    .wrap(JwtAuth)
                    .wrap(Permission::new("view_payment_transaction".to_string()))
                    .route(web::post().to(payments::show)),
            )
            .service(
                web::resource("/create")
                    .wrap(JwtAuth)
                    .wrap(Permission::new("create_payment_transaction".to_string()))
                    .route(web::post().to(payments::create)),
            )
            .service(
                web::resource("/retry/{pid}")
                    .wrap(JwtAuth)
                    .wrap(Permission::new("retry_payment_transaction".to_string()))
                    .route(web::post().to(payments::retry_payment)),
            )
            .service(
                web::resource(format!("/callbacks/mpesa/{}", secret))
                    .route(web::post().to(payments::mpesa_callback)),
            )
            .service(
                web::resource(format!("/webhooks/paypal/{}", secret))
                    .route(web::post().to(payments::paypal_webhook)),
            )
            .service(
                web::resource(format!("/webhooks/stripe/{}", secret))
                    .route(web::post().to(payments::stripe_webhook)),
            ),
    );
}
