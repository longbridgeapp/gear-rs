use std::io;

use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use poem::{
    endpoint::BoxEndpoint,
    listener::TcpListener,
    middleware::{OpenTelemetryMetrics, OpenTelemetryTracing, TokioMetrics},
    EndpointExt, IntoEndpoint, Response, Server,
};
use poem_grpc::{RouteGrpc, Service};

use crate::middlewares::SetCurrentService;

/// GRPC Server
#[derive(Default)]
pub struct GrpcServer {
    router: RouteGrpc,
}

impl GrpcServer {
    /// Create a GRPC server
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a GRPC service
    pub fn add_service<S>(mut self, service: S) -> Self
    where
        S: IntoEndpoint<Endpoint = BoxEndpoint<'static, Response>> + Service,
    {
        self.router = self.router.add_service(service);
        self
    }

    /// Start the server
    pub async fn start(self) -> io::Result<()> {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let tracer_provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("Trace Pipeline should initialize.");
        let tracer = tracer_provider.tracer("gear-rs");
        let grpc_server = Server::new(TcpListener::bind(
            std::env::var("MICRO_SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
        ))
        .http2_max_concurrent_streams(None)
        .run({
            self.router
                .data(tracer.clone())
                .with(OpenTelemetryTracing::new(tracer))
                .with(OpenTelemetryMetrics::new())
                .with(SetCurrentService)
                .with_if(
                    std::env::var("GEAR_ENABLE_TOKIO_METRICS").as_deref() == Ok("1"),
                    TokioMetrics::new(),
                )
        });

        tokio::try_join!(grpc_server).map(|_| ())
    }
}
