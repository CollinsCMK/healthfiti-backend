use crate::{middlewares::jwt_auth::JwtAuth, routes};
use actix_web::web::{self, ServiceConfig};

pub fn config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/admin")
            .wrap(JwtAuth)
            .configure(routes::admin::patients::config)
            .configure(routes::admin::patient_insurance::config)
            .configure(routes::admin::tenants::config)
            .configure(routes::admin::tenant_applications::config)
            .configure(routes::admin::users::config)
            .configure(routes::admin::subscription_plans::config)
            .configure(routes::admin::payments::config)
            .configure(routes::admin::subscriptions::config)
            .configure(routes::admin::billing_line_items::config),
    );
}
