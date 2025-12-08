use crate::routes;
use actix_web::web::{self, ServiceConfig};

pub fn config(config: &mut ServiceConfig) {
    config.service(web::scope("/tenant").configure(routes::tenant::tenants::config));
}
