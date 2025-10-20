use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, error::ErrorForbidden,
    http::Method,
};
use futures_util::future::LocalBoxFuture;
use std::{future::{ready, Ready}, rc::Rc};
use infrastructure_csrf::CsrfToken;
use tracing::warn;

#[derive(Debug)]
pub struct CsrfMiddleware {
    secret: String,
}

impl CsrfMiddleware {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CsrfMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CsrfMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CsrfMiddlewareService {
            service: Rc::new(service),
            secret: self.secret.clone(),
        }))
    }
}

#[derive(Debug)]
pub struct CsrfMiddlewareService<S> {
    service: Rc<S>,
    secret: String,
}

impl<S, B> Service<ServiceRequest> for CsrfMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let secret = self.secret.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // Only check POST, PUT, DELETE
            if matches!(req.method(), &Method::POST | &Method::PUT | &Method::DELETE) {
                let session_token = req
                    .cookie("session")
                    .map(|c| c.value().to_string())
                    .ok_or_else(|| ErrorForbidden("No session"))?;

                let csrf_token = req
                    .headers()
                    .get("X-CSRF-Token")
                    .and_then(|h| h.to_str().ok())
                    .ok_or_else(|| ErrorForbidden("No CSRF token"))?;

                if !CsrfToken::validate(csrf_token, &session_token, &secret) {
                    warn!("CSRF validation failed");
                    return Err(ErrorForbidden("Invalid CSRF token"));
                }
            }

            service.call(req).await
        })
    }
}