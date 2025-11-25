use actix_web::web::{self};

use crate::handlers::auth::login::admin_login;

pub fn config(config: &mut web::ServiceConfig) {
    config.service(web::scope("/auth").service(admin_login));
}
