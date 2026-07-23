use std::env;

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use payrail::{
    PayRailOtelMetrics, PaymentEventType, PaymentProvider, ProviderTelemetryOperation,
    TelemetryOperation, TelemetryResult,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = init_application_tracing()?;

    let checkout_span = tracing::info_span!(
        "payrail.example.checkout",
        "payrail.operation" = TelemetryOperation::PaymentCreate.as_str(),
        "payrail.provider" = "stripe"
    );

    {
        let _guard = checkout_span.enter();
        tracing::info!(
            "payrail.operation" = TelemetryOperation::PaymentCreate.as_str(),
            "payrail.result" = TelemetryResult::Ok.as_str(),
            "example payment span emitted"
        );
    }

    // Applications also own metric SDK/exporter setup. Without a configured meter provider, these
    // OpenTelemetry API instruments are no-ops.
    let meter = global::meter("payrail.example");
    let metrics = PayRailOtelMetrics::new(&meter);
    metrics.record_payment_request(
        Some(&PaymentProvider::Stripe),
        TelemetryOperation::PaymentCreate,
        TelemetryResult::Ok,
    );
    metrics.record_provider_request(
        &PaymentProvider::Stripe,
        ProviderTelemetryOperation::CreatePayment,
        TelemetryResult::Ok,
    );
    metrics.record_provider_request_duration_ms(
        &PaymentProvider::Stripe,
        ProviderTelemetryOperation::CreatePayment,
        TelemetryResult::Ok,
        42.0,
    );
    metrics.record_webhook(
        &PaymentProvider::Stripe,
        Some(PaymentEventType::PaymentSucceeded),
        TelemetryResult::Ok,
    );

    provider.shutdown()?;
    Ok(())
}

fn init_application_tracing() -> Result<SdkTracerProvider, Box<dyn std::error::Error>> {
    let endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4318/v1/traces".to_owned());
    let service_name =
        env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "payrail-example".to_owned());
    let environment =
        env::var("DEPLOYMENT_ENVIRONMENT").unwrap_or_else(|_| "development".to_owned());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(endpoint)
        .build()?;
    let resource = Resource::builder()
        .with_service_name(service_name)
        .with_attributes([
            KeyValue::new("service.namespace", "payrail"),
            KeyValue::new("deployment.environment", environment),
        ])
        .build();
    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_simple_exporter(exporter)
        .build();
    let tracer = provider.tracer("payrail.example");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(otel_layer)
        .init();

    Ok(provider)
}
