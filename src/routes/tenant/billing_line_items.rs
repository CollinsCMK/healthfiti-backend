use actix_web::web::{self};

use crate::{handlers::tenant::billing_line_items, middlewares::permissions::Permission};

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
            ),
    );
}
