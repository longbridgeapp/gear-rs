use std::io;

use opentelemetry::{global, sdk::propagation::TraceContextPropagator};
use poem::{
    endpoint::{BoxEndpoint, PrometheusExporter},
    get,
    listener::TcpListener,
    middleware::{OpenTelemetryMetrics, OpenTelemetryTracing},
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
        let tracer = opentelemetry_jaeger::new_agent_pipeline()
            .install_batch(opentelemetry::runtime::Tokio)
            .unwrap();

        let prometheus_exporter_server = Server::new(TcpListener::bind("0.0.0.0:9102"))
            .run(poem::Route::new().at("/metrics", get(PrometheusExporter::new())));

        let grpc_server = Server::new(TcpListener::bind(
            std::env::var("MICRO_SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
        ))
        .run(
            self.router
                .data(tracer.clone())
                .with(OpenTelemetryTracing::new(tracer))
                .with(OpenTelemetryMetrics::new())
                .with(SetCurrentService),
        );

        tokio::try_join!(prometheus_exporter_server, grpc_server).map(|_| ())
    }
}
