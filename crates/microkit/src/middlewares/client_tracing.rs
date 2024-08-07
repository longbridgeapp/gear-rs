use opentelemetry::{
    global,
    trace::{FutureExt, Span, SpanKind, TraceContextExt, Tracer as _},
    Context, KeyValue,
};
use opentelemetry_http::HeaderInjector;
use opentelemetry_sdk::trace::Tracer;
use opentelemetry_semantic_conventions::trace;
use poem::{Endpoint, Middleware, Request, Result};

pub struct ClientTracing;

impl<E: Endpoint> Middleware<E> for ClientTracing {
    type Output = ClientTracingEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        ClientTracingEndpoint { inner: ep }
    }
}

pub struct ClientTracingEndpoint<E> {
    inner: E,
}

impl<E: Endpoint> Endpoint for ClientTracingEndpoint<E> {
    type Output = E::Output;

    async fn call(&self, mut req: Request) -> Result<Self::Output> {
        match req.data::<Tracer>() {
            Some(tracer) => {
                let mut span = tracer
                    .span_builder("grpc request")
                    .with_kind(SpanKind::Client)
                    .start(tracer);
                span.set_attribute(KeyValue::new(trace::URL_FULL, req.uri().path().to_string()));

                let cx = Context::current_with_span(span);
                global::get_text_map_propagator(|propagator| {
                    propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut()))
                });

                self.inner.call(req).with_context(cx).await
            }
            None => self.inner.call(req).await,
        }
    }
}
