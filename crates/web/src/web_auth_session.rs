use actix_web::{
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse, Service, Transform},
    Error, HttpMessage, HttpResponse,
};
use std::future::{ready, Ready, Future};
use std::pin::Pin;
use std::rc::Rc;

/// Session information attached to requests
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub session_id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub user_email: String,
}

/// Middleware that requires authentication
#[derive(Debug, Clone, Copy)]  // ← ADD Copy and Debug
pub struct RequireAuth;

impl<S> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = RequireAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

/// The actual middleware implementation
#[derive(Debug)]  // ← ADD Debug
pub struct RequireAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for RequireAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Step 1: Extract session_id from cookie
            let session_id = match extract_session_cookie(&req) {
                Some(id) => id,
                None => {
                    tracing::warn!("No session cookie found, redirecting to login");
                    return Ok(redirect_to_login(req));
                }
            };

            tracing::debug!("Found session cookie: {}", session_id);

            // Step 2: Get app state
            let app_state = match req.app_data::<actix_web::web::Data<crate::AppState>>() {
                Some(s) => s.clone(),
                None => {
                    tracing::error!("Failed to get app state");
                    return Ok(redirect_to_login(req));
                }
            };

            // Step 3: Fetch session from Redis
            let mut redis_conn = app_state.redis.clone();
            let session = match infra::fetch_session_by_id(&mut redis_conn, &session_id).await {
                Ok(Some(s)) => s,
                Ok(None) => {
                    tracing::warn!("Session not found in Redis: {}", session_id);
                    return Ok(redirect_to_login(req));
                }
                Err(e) => {
                    tracing::error!("Redis error fetching session: {:?}", e);
                    return Ok(redirect_to_login(req));
                }
            };

            // Step 4: Fetch user info from database
            let user = match infra::find_by_id(&app_state.db, &session.user_id).await {
                Ok(Some(u)) => u,
                Ok(None) => {
                    tracing::warn!("User not found: {}", session.user_id);
                    return Ok(redirect_to_login(req));
                }
                Err(e) => {
                    tracing::error!("Database error fetching user: {:?}", e);
                    return Ok(redirect_to_login(req));
                }
            };

            tracing::info!("Authenticated user: {}", user.email);

            // Step 5: Attach session info to request extensions
            let auth_session = AuthSession {
                session_id: session.id,
                user_id: user.id.clone(),
                tenant_id: user.tenant_id,
                user_email: user.email,
            };

            let _ = req.extensions_mut().insert(auth_session);  // ← ADD let _ =

            // Step 6: Continue to the handler
            service.call(req).await
        })
    }
}

/// Extract session_id from cookies
fn extract_session_cookie(req: &ServiceRequest) -> Option<String> {
    req.cookie("session_id").map(|cookie| cookie.value().to_string())
}

/// Redirect to login page
fn redirect_to_login(req: ServiceRequest) -> ServiceResponse<BoxBody> {
    let (request, _pl) = req.into_parts();
    let response = HttpResponse::Found()
        .insert_header(("Location", "/login"))
        .finish()
        .map_into_boxed_body();

    ServiceResponse::new(request, response)
}