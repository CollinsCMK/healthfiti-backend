use crate::routes;
use actix_web::web::{self, ServiceConfig};

pub fn config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/admin")
            .configure(routes::admin::patients::config)
            .configure(routes::admin::patient_insurance::config),
    );
}
