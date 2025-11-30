use actix_web::web::{self};

use crate::handlers::health::health;

pub fn config(config: &mut web::ServiceConfig) {
    config.service(web::scope("/patients").service(health));
}
