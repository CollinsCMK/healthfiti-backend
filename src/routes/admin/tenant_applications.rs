use actix_web::web::{self};

use crate::{handlers::admin::tenant_applications, middlewares::permissions::Permission};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/tenant_applications")
            .service(
                web::resource("")
                    .wrap(Permission::new("view_tenant_applications".to_string()))
                    .route(web::get().to(tenant_applications::index)),
            )
            .service(
                web::resource("/show/{tenant_id}")
                    .wrap(Permission::new("view_tenant_application".to_string()))
                    .route(web::get().to(tenant_applications::show)),
            )
            .service(
                web::resource("/create")
                    .wrap(Permission::new("create_tenant_application".to_string()))
                    .route(web::post().to(tenant_applications::create)),
            )
            .service(
                web::resource("/edit/{tenant_id}")
                    .wrap(Permission::new("update_tenant_application".to_string()))
                    .route(web::put().to(tenant_applications::update)),
            )
            .service(
                web::resource("/delete/{tenant_id}")
                    .wrap(Permission::new("soft_delete_tenant_application".to_string()))
                    .route(web::delete().to(tenant_applications::destroy)),
            )
            .service(
                web::resource("/restore/{tenant_id}")
                    .wrap(Permission::new("restore_tenant_application".to_string()))
                    .route(web::put().to(tenant_applications::restore)),
            )
            .service(
                web::resource("/permanent/{tenant_id}")
                    .wrap(Permission::new("delete_tenant_application".to_string()))
                    .route(web::delete().to(tenant_applications::delete_permanently)),
            ),
    );
}
