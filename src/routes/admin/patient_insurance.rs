use actix_web::web::{self};

use crate::{handlers::admin, middlewares::permissions::Permission};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/patient_insurance")
            .service(
                web::resource("")
                    .wrap(Permission::new("view_all_patient_insurances".to_string()))
                    .route(web::get().to(admin::patients::insurance::index)),
            )
            .service(
                web::resource("/show/{pid}")
                    .wrap(Permission::new("view_patient_insurance".to_string()))
                    .route(web::get().to(admin::patients::insurance::show)),
            )
            .service(
                web::resource("/create")
                    .wrap(Permission::new("create_patient_insurance".to_string()))
                    .route(web::post().to(admin::patients::insurance::create)),
            )
            .service(
                web::resource("/edit/{pid}")
                    .wrap(Permission::new("update_patient_insurance".to_string()))
                    .route(web::put().to(admin::patients::insurance::edit)),
            )
            .service(
                web::resource("/primary/{pid}")
                    .wrap(Permission::new(
                        "update_primary_patient_insurance".to_string(),
                    ))
                    .route(web::patch().to(admin::patients::insurance::set_primary)),
            )
            .service(
                web::resource("/permanent/{pid}")
                    .wrap(Permission::new("delete_patient_insurance".to_string()))
                    .route(web::delete().to(admin::patients::insurance::delete_permanently)),
            )
            .service(
                web::resource("/soft-delete/{pid}")
                    .wrap(Permission::new("soft_delete_patient_insurance".to_string()))
                    .route(web::delete().to(admin::patients::insurance::destroy)),
            )
            .service(
                web::resource("/restore/{pid}")
                    .wrap(Permission::new("restore_patient_insurance".to_string()))
                    .route(web::patch().to(admin::patients::insurance::restore)),
            ),
    );
}
