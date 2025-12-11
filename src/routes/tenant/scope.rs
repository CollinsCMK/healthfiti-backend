use crate::{middlewares::jwt_auth::JwtAuth, routes};
use actix_web::web::{self, ServiceConfig};

pub fn config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/tenant")
            .service(
                web::scope("")
                    .wrap(JwtAuth)
                    .configure(routes::tenant::tenants::config)
                    .configure(routes::tenant::users::config)
                    .configure(routes::tenant::subscription_plans::config)
            )
            .configure(routes::tenant::payments::config),
    );
}
