use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use serde_json::json;
use std::future::{Ready, ready};

use crate::utils::{api_response::ApiResponse, jwt::decode_token};

pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware { service }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
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
        // ------------------------------------
        // 1. Extract token BEFORE moving req
        // ------------------------------------
        let auth_header = req.headers().get("Authorization");

        let token = match auth_header {
            Some(auth_value) => match auth_value.to_str() {
                Ok(auth_str) => {
                    if auth_str.starts_with("Bearer ") {
                        &auth_str[7..]
                    } else {
                        return Box::pin(async {
                            Err(ApiResponse::new(
                                401,
                                json!({ "message": "Invalid authorization header format. Expected 'Bearer <token>'" }).into()
                            ).into())
                        });
                    }
                }
                Err(_) => {
                    return Box::pin(async {
                        Err(ApiResponse::new(
                            401,
                            json!({ "message": "Invalid authorization header encoding" }).into(),
                        )
                        .into())
                    });
                }
            },
            None => {
                return Box::pin(async {
                    Err(ApiResponse::new(
                        401,
                        json!({ "message": "Missing authorization header" }).into(),
                    )
                    .into())
                });
            }
        };

        // --------------------------
        // 2. Decode token safely
        // --------------------------
        let claims = match decode_token(token) {
            Ok(c) => c,
            Err(e) => {
                return Box::pin(async move {
                    Err(ApiResponse::new(
                        401,
                        json!({ "message": format!("Failed to decode token: {}", e) }).into(),
                    )
                    .into())
                });
            }
        };

        // Insert claims before passing to service
        req.extensions_mut().insert(claims);

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
