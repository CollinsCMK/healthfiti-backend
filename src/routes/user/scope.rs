use crate::routes;
use actix_web::web::{self, ServiceConfig};

pub fn config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/me")
            .configure(routes::user::profile::config)
            .configure(routes::user::insurance::config)
            .configure(routes::user::tenants::config),
    );
}
