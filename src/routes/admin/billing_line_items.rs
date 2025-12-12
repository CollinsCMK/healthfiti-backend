use actix_web::web::{self};

use crate::{handlers::admin::billing_line_items, middlewares::permissions::Permission};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/billing-line-items")
            .service(
                web::resource("")
                    .wrap(Permission::new("view_all_billing_line_items".to_string()))
                    .route(web::get().to(billing_line_items::index)),
            )
            .service(
                web::resource("/show/{pid}")
                    .wrap(Permission::new("view_billing_line_item".to_string()))
                    .route(web::get().to(billing_line_items::show)),
            )
            .service(
                web::resource("/destroy/{pid}")
                    .wrap(Permission::new(
                        "soft_delete_billing_line_item".to_string(),
                    ))
                    .route(web::delete().to(billing_line_items::destroy)),
            )
            .service(
                web::resource("/restore/{pid}")
                    .wrap(Permission::new(
                        "restore_billing_line_item".to_string(),
                    ))
                    .route(web::post().to(billing_line_items::restore)),
            )
            .service(
                web::resource("/permanent/{pid}")
                    .wrap(Permission::new(
                        "permanent_delete_billing_line_item".to_string(),
                    ))
                    .route(web::delete().to(billing_line_items::delete_permanently)),
            ),
    );
}
