use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, error::ErrorUnauthorized,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    sync::Arc,
    rc::Rc,
};
use tracing::{instrument, warn, info};
use infrastructure_session::RedisSessionStore;

#[derive(Debug)]
pub struct AuthMiddleware {
    store: Arc<RedisSessionStore>,
}

impl AuthMiddleware {
    pub fn new(store: Arc<RedisSessionStore>) -> Self {
        Self { store }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            store: self.store.clone(),
        }))
    }
}

#[derive(Debug)]
pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    store: Arc<RedisSessionStore>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    #[instrument(skip(self, req))]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let store = self.store.clone();
        let service = self.service.clone();

        Box::pin(async move {
            let token = req
                .cookie("session")
                .map(|c| c.value().to_string())
                .ok_or_else(|| ErrorUnauthorized("No session"))?;

            store.validate_token(&token).await
                .map_err(|_| ErrorUnauthorized("Invalid token"))?;

            let session = store.get(&token).await
                .map_err(|e| {
                    warn!("Session lookup failed: {:?}", e);
                    ErrorUnauthorized("Session not found")
                })?;

            // Validate tenant isolation
            let host = req.connection_info().host().to_string();
            if !host.contains(&session.tenant_id) {
                warn!("Tenant mismatch: {} vs {}", host, session.tenant_id);
                return Err(ErrorUnauthorized("Tenant mismatch"));
            }

            // Refresh TTL (sliding expiration)
            if let Err(e) = store.refresh(&token, 1800).await {
                warn!("Failed to refresh session: {:?}", e);
            }

            info!("Session validated for user: {}", session.user_id);
            req.extensions_mut().insert(session);

            service.call(req).await
        })
    }
}