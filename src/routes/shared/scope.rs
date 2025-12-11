use crate::{middlewares::jwt_auth::JwtAuth, routes};
use actix_web::web::{self, ServiceConfig};

pub fn config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/shared")
            .wrap(JwtAuth)
            .configure(routes::shared::profile::config)
            .configure(routes::shared::session::config)
            .configure(routes::shared::test::config),
    );
}
