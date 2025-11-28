use crate::routes;
use actix_web::web::{self, ServiceConfig};

pub fn config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/tenant")
            .configure(routes::shared::profile::config)
            // .configure(routes::shared::sessions::config)
            // .configure(routes::shared::test::config),
    );
}