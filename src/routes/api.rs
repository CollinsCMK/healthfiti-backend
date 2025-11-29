use actix_web::web::{self, ServiceConfig};

use crate::{middlewares::jwt_auth::JwtAuth, routes};

pub fn config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/api").configure(routes::auth::config).service(
            web::scope("")
                .wrap(JwtAuth)
                .configure(routes::public::scope::config)
                .configure(routes::user::scope::config)
                .configure(routes::tenant::scope::config)
                .configure(routes::admin::scope::config)
                .configure(routes::shared::scope::config),
        ),
    );
}
