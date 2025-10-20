use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::{future::{ready, Ready}, rc::Rc};

#[derive(Debug, Clone, Copy)]
pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SecurityHeadersService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersService {
            service: Rc::new(service),
        }))
    }
}

#[derive(Debug)]
pub struct SecurityHeadersService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersService<S>
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
        let service = self.service.clone();

        Box::pin(async move {
            let mut res = service.call(req).await?;

            let headers = res.headers_mut();
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-frame-options"),
                actix_web::http::header::HeaderValue::from_static("DENY"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("strict-transport-security"),
                actix_web::http::header::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                actix_web::http::header::HeaderValue::from_static("1; mode=block"),
            );

            Ok(res)
        })
    }
}