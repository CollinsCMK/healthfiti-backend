use actix_web::web::{self};

use crate::handlers::user::profile::{
    emergency_information, health_information, personal_information,
};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/profile")
            .service(personal_information::get_personal_information)
            .service(personal_information::upsert)
            .service(health_information::get_health_information)
            .service(health_information::upsert)
            .service(emergency_information::get_emergency_information)
            .service(emergency_information::upsert),
    );
}
