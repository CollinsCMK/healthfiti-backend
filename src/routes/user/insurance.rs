use actix_web::web::{self};

use crate::handlers::user::profile::insurance;

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/insurance")
            .service(insurance::index)
            .service(insurance::show)
            .service(insurance::create)
            .service(insurance::edit)
            .service(insurance::set_primary)
            .service(insurance::destroy)
    );
}
