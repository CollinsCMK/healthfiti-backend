use actix_web::web::{self};

use crate::handlers::shared;

pub fn config(config: &mut web::ServiceConfig) {
    config.service(web::scope("/test").service(shared::test::send));
}
