use opentelemetry::{trace::TracerProvider, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{MetricExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider, Temporality},
    propagation::TraceContextPropagator,
    trace::{RandomIdGenerator, Sampler},
    Resource,
};
use std::{io::IsTerminal, net::SocketAddr, sync::OnceLock, time::Duration};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

const DEFAULT_INTERVAL: Duration = Duration::from_secs(1);
static TRACING_INITIALIZER: OnceLock<()> = OnceLock::new();

pub fn init_opentelemetry(endpoint: &SocketAddr, resource: Resource) {
    let resource = Resource::builder()
        .with_attributes(
            resource
                .iter()
                .map(|(k, v)| KeyValue::new(k.clone(), v.clone())),
        )
        .with_attribute(KeyValue::new(
            "host.name",
            hostname::get()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_owned(),
        ))
        .with_attribute(KeyValue::new(
            "deployment.environment",
            option_env!("ROCKET_CLUSTER").unwrap_or("test"),
        ))
        .build();

    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_temporality(Temporality::Delta)
        .with_endpoint(format!("http://{endpoint}/v1/metrics"))
        .with_timeout(std::time::Duration::from_secs(10))
        .with_protocol(Protocol::Grpc)
        .build()
        .expect("Failed to create metric exporter");

    let reader = PeriodicReader::builder(exporter)
        .with_interval(DEFAULT_INTERVAL)
        .build();

    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource.clone())
        .build();

    opentelemetry::global::set_meter_provider(meter_provider);

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{endpoint}"))
        .with_timeout(Duration::from_secs(10))
        .with_protocol(Protocol::Grpc)
        .build()
        .expect("Failed to create tracer exporter");

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(64)
        .with_max_attributes_per_span(16)
        .with_resource(resource.clone())
        .build();

    let tracer = tracer_provider.tracer("root");
    opentelemetry::global::set_tracer_provider(tracer_provider);
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{endpoint}/v1/metrics"))
        .with_timeout(Duration::from_secs(10))
        .with_protocol(Protocol::Grpc)
        .build()
        .expect("Failed to create logs exporter");

    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource.clone())
        .build();

    TRACING_INITIALIZER.get_or_init(|| {
        let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

        let filter_otel = EnvFilter::new("info")
            .add_directive("hyper=off".parse().unwrap())
            .add_directive("opentelemetry=off".parse().unwrap())
            .add_directive("tonic=off".parse().unwrap())
            .add_directive("h2=off".parse().unwrap())
            .add_directive("reqwest=off".parse().unwrap());
        let logger_layer = logger_layer.with_filter(filter_otel);

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_thread_names(true)
            .with_ansi(std::io::stdout().is_terminal())
            .with_filter(EnvFilter::new("info"));

        let tracing_layer = OpenTelemetryLayer::new(tracer);
        let filter_otel = EnvFilter::new("info")
            .add_directive("hyper=off".parse().unwrap())
            .add_directive("opentelemetry=off".parse().unwrap())
            .add_directive("tonic=off".parse().unwrap())
            .add_directive("h2=off".parse().unwrap())
            .add_directive("reqwest=off".parse().unwrap());
        let tracing_layer = tracing_layer.with_filter(filter_otel);

        tracing_subscriber::registry()
            .with(logger_layer)
            .with(tracing_layer)
            .with(fmt_layer)
            .init();
    });
}
