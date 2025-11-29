use actix_web::web::{self};

use crate::handlers::shared;

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/profile")
            .service(shared::profile::get_logged_in_user_data)
            .service(shared::profile::enable_2fa_totp)
            .service(shared::profile::edit_2fa_totp)
            .service(shared::profile::verify_totp_data)
            .service(shared::profile::regenerate_recovery_codes)
            .service(shared::profile::enable_otp),
    );
}
