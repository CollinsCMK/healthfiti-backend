use actix_web::web::{self};

use crate::{
    handlers::auth::{
        email_verification::verify_email,
        login::{admin_login, normal_login, tenant_login},
        logout::logout,
        password_reset::reset_password,
        password_reset_request::{
            admin_password_reset_request, normal_password_reset_request,
            tenant_password_reset_request,
        },
        phone_verification::verify_phone,
        refresh::refresh,
        register::register,
        resend_verification_code::resend_otp,
        two_factor::login_verify,
    },
    middlewares::jwt_auth::JwtAuth,
};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/auth")
            .service(register)
            .service(verify_email)
            .service(verify_phone)
            .service(admin_login)
            .service(tenant_login)
            .service(normal_login)
            .service(resend_otp)
            .service(refresh)
            .service(login_verify)
            .service(admin_password_reset_request)
            .service(tenant_password_reset_request)
            .service(normal_password_reset_request)
            .service(reset_password)
            .service(web::scope("").wrap(JwtAuth).service(logout)),
    );
}
