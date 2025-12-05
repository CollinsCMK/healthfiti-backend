use actix_web::web::{self};

use crate::{handlers::tenant::tenants, middlewares::permissions::Permission};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/me")
        .service(
            web::resource("")
                .wrap(Permission::new("view_tenant".to_string()))
                .route(web::get().to(tenants::show)),
        )
        .service(
            web::resource("")
                .wrap(Permission::new("update_tenant".to_string()))
                .route(web::put().to(tenants::update)),
        )
    );
}
