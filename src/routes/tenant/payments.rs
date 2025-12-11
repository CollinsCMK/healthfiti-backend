use actix_web::web::{self};

use crate::{handlers::tenant::payments, middlewares::permissions::Permission, utils};

pub fn config(config: &mut web::ServiceConfig) {
    let secret = (utils::constants::SECRET).clone();

    config.service(
        web::scope("/payments")
            .service(
                web::resource("")
                    .route(web::get().to(payments::index)),
            )
            .service(
                web::resource("/show/{pid}")
                    .route(web::post().to(payments::show)),
            )
            .service(
                web::resource("/create")
                    .route(web::post().to(payments::create)),
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
