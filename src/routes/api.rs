use actix_web::web::{self, ServiceConfig};

use crate::routes;

pub fn config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/api").configure(routes::auth::config), // .service()
    );
}
