use actix_web::web::{self};

use crate::handlers::user::tenants;

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/tenants")
        .service(tenants::index)
        .service(tenants::show)
    );
}
