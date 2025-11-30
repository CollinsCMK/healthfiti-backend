use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use serde_json::json;
use std::future::{Ready, ready};

use crate::utils::{api_response::ApiResponse, permission::extract_permissions};

pub struct Permission {
    guard_names: Vec<String>,
}

impl Permission {
    // Constructor for single permission
    pub fn new(guard_name: String) -> Self {
        Self {
            guard_names: vec![guard_name],
        }
    }

    // Constructor for multiple permissions
    pub fn with_permissions(guard_names: Vec<String>) -> Self {
        Self { guard_names }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Permission
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PermissionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PermissionMiddleware {
            service,
            guard_names: self.guard_names.clone(),
        }))
    }
}

pub struct PermissionMiddleware<S> {
    service: S,
    guard_names: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for PermissionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let guard_names = self.guard_names.clone();
        let req_clone = req.request().clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            for guard_name in guard_names.iter() {
                let has_permission = match extract_permissions(guard_name.clone(), &req_clone).await
                {
                    Ok(permission) => permission,
                    Err(_) => false,
                };

                if has_permission {
                    let res = fut.await?;
                    return Ok(res);
                }
            }

            Err(ApiResponse::new(
                401,
                json!({
                    "message": "Permission denied. You do not have the required access rights for this action.",
                    "required_permissions": guard_names,
                }),
            )
            .into())
        })
    }
}
