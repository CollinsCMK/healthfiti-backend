use actix_web::web::{self};

use crate::{handlers::tenant::users, middlewares::permissions::Permission};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/tenant_users")
            .service(
                web::resource("")
                    .wrap(Permission::new("view_all_users".to_string()))
                    .route(web::get().to(users::index)),
            )
            .service(
                web::resource("/show/{user_id}")
                    .wrap(Permission::new("view_user".to_string()))
                    .route(web::get().to(users::show)),
            )
            .service(
                web::resource("/create")
                    .wrap(Permission::new("create_user".to_string()))
                    .route(web::post().to(users::create)),
            )
            .service(
                web::resource("/edit/{user_id}")
                    .wrap(Permission::new("update_user".to_string()))
                    .route(web::put().to(users::edit)),
            )
            .service(
                web::resource("/status/{user_id}")
                    .wrap(Permission::new("activate_or_deactivate_user".to_string()))
                    .route(web::patch().to(users::set_active_status)),
            )
            .service(
                web::resource("/destroy/{user_id}")
                    .wrap(Permission::new("soft_delete_user".to_string()))
                    .route(web::delete().to(users::destroy)),
            )
            .service(
                web::resource("/restore/{user_id}")
                    .wrap(Permission::new("restore_user".to_string()))
                    .route(web::put().to(users::restore)),
            )
            .service(
                web::resource("/permanent/{user_id}")
                    .wrap(Permission::new("delete_user".to_string()))
                    .route(web::delete().to(users::delete_permanently)),
            ),
    );
}
