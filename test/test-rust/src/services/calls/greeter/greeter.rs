/// A simple greeter service.
pub trait Greeter: Send + Sync {
    /// Returns a greeting for the given name.
    fn greet(
        &self,
        request: crate::services::calls::GreetRequest,
    ) -> impl ::std::future::Future<
        Output = ::std::result::Result<
            crate::services::calls::GreetResponse,
            ::proto_packet::service::ServiceError,
        >,
    > + Send
    + '_;
}

pub fn router<S>(service: ::std::sync::Arc<S>) -> ::axum::Router
where
    S: Greeter + 'static,
{
    ::axum::Router::new().route(
        "/services.calls.Greeter/greet",
        ::axum::routing::post({
            let service = service.clone();
            move |::axum::Json(request): ::axum::Json<crate::services::calls::GreetRequest>| {
                let service = service.clone();
                async move { service.greet(request).await.map(::axum::Json) }
            }
        }),
    )
}
