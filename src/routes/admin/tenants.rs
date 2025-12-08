use actix_web::web::{self};

use crate::{handlers::admin::tenants, middlewares::permissions::Permission};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/tenants")
            .service(
                web::resource("")
                    .wrap(Permission::with_permissions(vec![
                        "manage_all_tenants".to_string(),
                        "view_all_tenants".to_string(),
                    ]))
                    .route(web::get().to(tenants::index)),
            )
            .service(
                web::resource("/show/{tenant_id}")
                    .wrap(Permission::new("view_tenant".to_string()))
                    .route(web::get().to(tenants::show)),
            )
            .service(
                web::resource("/create")
                    .wrap(Permission::new("create_tenant".to_string()))
                    .route(web::post().to(tenants::create)),
            )
            .service(
                web::resource("/edit/{tenant_id}")
                    .wrap(Permission::new("update_tenant".to_string()))
                    .route(web::put().to(tenants::update)),
            )
            .service(
                web::resource("/status/{tenant_id}")
                    .wrap(Permission::new("activate_or_deactivate_tenant".to_string()))
                    .route(web::patch().to(tenants::set_active_status)),
            )
            .service(
                web::resource("/delete/{tenant_id}")
                    .wrap(Permission::new("soft_delete_tenant".to_string()))
                    .route(web::delete().to(tenants::destroy)),
            )
            .service(
                web::resource("/restore/{tenant_id}")
                    .wrap(Permission::new("restore_tenant".to_string()))
                    .route(web::put().to(tenants::restore)),
            )
            .service(
                web::resource("/permanent/{tenant_id}")
                    .wrap(Permission::new("delete_tenant".to_string()))
                    .route(web::delete().to(tenants::delete_permanently)),
            ),
    );
}
