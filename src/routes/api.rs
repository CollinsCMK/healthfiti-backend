use actix_web::web::{self, ServiceConfig};

use crate::routes;

pub fn config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/api")
            .configure(routes::auth::config)
            .configure(routes::public::scope::config)
            .configure(routes::user::scope::config)
            .configure(routes::tenant::scope::config)
            .configure(routes::admin::scope::config)
            .configure(routes::shared::scope::config),
    );
}
