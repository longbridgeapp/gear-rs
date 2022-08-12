#![allow(clippy::declare_interior_mutable_const)] // https://github.com/tokio-rs/tokio/issues/4872#issuecomment-1197753788

use poem::{Endpoint, Middleware, Request, Result};

pub(crate) struct SetCurrentService;

impl<E: Endpoint> Middleware<E> for SetCurrentService {
    type Output = SetCurrentServiceEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        SetCurrentServiceEndpoint { inner: ep }
    }
}

tokio::task_local! {
    static SERVICE_NAME: String;
}

pub(crate) fn current_service_name() -> String {
    SERVICE_NAME.with(|service_name| service_name.clone())
}

pub(crate) struct SetCurrentServiceEndpoint<E> {
    inner: E,
}

#[poem::async_trait]
impl<E: Endpoint> Endpoint for SetCurrentServiceEndpoint<E> {
    type Output = E::Output;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        // x-micro-service
        if let Some(service) = req.uri().path().split('/').rev().nth(1) {
            SERVICE_NAME
                .scope(service.to_string(), self.inner.call(req))
                .await
        } else {
            self.inner.call(req).await
        }
    }
}
