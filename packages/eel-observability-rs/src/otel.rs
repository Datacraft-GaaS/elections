use crate::error::ObservabilityError;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::TracerProvider;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Configuration for OTel exporters.
#[derive(Debug, Clone)]
pub struct OtelConfig {
    /// OTLP endpoint for traces (gRPC). Default: OTEL_EXPORTER_OTLP_TRACES_ENDPOINT or localhost:4317
    pub traces_endpoint: String,
    /// OTLP endpoint for metrics. Default: OTEL_EXPORTER_OTLP_METRICS_ENDPOINT or localhost:4317
    pub metrics_endpoint: String,
    /// Service name.
    pub service_name: String,
}

impl OtelConfig {
    pub fn from_env(service_name: impl Into<String>) -> Self {
        Self {
            traces_endpoint: std::env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".into()),
            metrics_endpoint: std::env::var("OTEL_EXPORTER_OTLP_METRICS_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".into()),
            service_name: service_name.into(),
        }
    }
}

/// Guards the tracing subscriber lifetime.
pub struct OtelGuard {
    provider: TracerProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(e) = self.provider.shutdown() {
            eprintln!("OTel shutdown error: {e:?}");
        }
    }
}

/// Initialize OTel tracing with OTLP exporter and a PII-scrubbing subscriber layer.
///
/// Returns a guard that shuts down the provider on drop.
pub fn init_otel(config: &OtelConfig) -> Result<OtelGuard, ObservabilityError> {
    use opentelemetry::KeyValue;
    use opentelemetry_sdk::Resource;

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&config.traces_endpoint)
        .build()
        .map_err(|e| ObservabilityError::Init(e.to_string()))?;

    let resource = Resource::new(vec![KeyValue::new(
        "service.name",
        config.service_name.clone(),
    )]);

    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(resource)
        .build();

    global::set_tracer_provider(provider.clone());

    let otel_layer =
        tracing_opentelemetry::layer().with_tracer(provider.tracer(config.service_name.clone()));

    let pii_filter = crate::scrubber::PiiScrubberLayer;

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(otel_layer)
        .with(pii_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .try_init()
        .map_err(|e| ObservabilityError::Init(e.to_string()))?;

    Ok(OtelGuard { provider })
}
