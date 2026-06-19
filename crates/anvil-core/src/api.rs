use axum::{
    Router,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

pub use anvil_macros::*;

pub struct ApiEndpoint {
    pub register: fn(Router) -> Router,
}
inventory::collect!(ApiEndpoint);

pub fn build_router() -> Router {
    inventory::iter::<ApiEndpoint>
        .into_iter()
        .fold(Router::new(), |router, ep| (ep.register)(router))
}

pub struct Svc<T>(pub T);

impl<T, S> FromRequestParts<S> for Svc<T>
where
    T: Clone + Send + Sync + 'static,
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let value = parts.extensions.get::<T>().cloned();
        async move {
            value
                .map(Svc)
                .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "service not registered"))
        }
    }
}
