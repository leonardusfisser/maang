use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, //HttpResponse
};
use std::future::{ready, Ready};

#[derive(Debug, Clone, Copy)]
pub struct CsrfProtection;

impl<S, B> Transform<S, ServiceRequest> for CsrfProtection
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CsrfMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CsrfMiddleware { service }))
    }
}

#[derive(Debug)]
pub struct CsrfMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CsrfMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Only validate CSRF on POST, PUT, DELETE, PATCH
        if !matches!(req.method().as_str(), "POST" | "PUT" | "DELETE" | "PATCH") {
            return self.service.call(req);
        }

        // Skip CSRF for public routes (can be refined later)
        let path = req.path();
        if path.starts_with("/static") || path.starts_with("/assets") {
            return self.service.call(req);
        }

        // TODO: Implement actual CSRF validation
        // For now, just pass through - we'll add full validation later
        self.service.call(req)
    }
}