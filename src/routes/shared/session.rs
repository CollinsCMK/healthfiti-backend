use actix_web::web::{self};

use crate::handlers::shared;

pub fn config(config: &mut web::ServiceConfig) {
    config.service(web::scope("/session").service(shared::session::get_user_sessions));
}
