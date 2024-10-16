// pub mod example {
//     tonic::include_proto!("kuksa.val.v1"); // Automatically generated from message.proto
// }

mod kuksa {
    pub mod val {
        pub mod v1 {
            tonic::include_proto!("kuksa.val.v1"); // The string must match the proto package name
        }
    }
}

use kuksa::val::v1::datapoint::Value;
use kuksa::val::v1::Field;
use kuksa::val::v1::{val_client::ValClient, DataEntry, EntryUpdate, Metadata, SetRequest};
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::{trace, Resource};
use opentelemetry::trace::TraceError;
use opentelemetry::{global, runtime, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use prost_types::Timestamp;
use std::convert::TryFrom;
use tonic::{metadata::MetadataKey, metadata::MetadataMap, metadata::MetadataValue, Request};
use tracing::{info, span, Level};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::layer::SubscriberExt;

// Custom Injector for gRPC MetadataMap
struct MetadataMapInjector<'a>(&'a mut MetadataMap);

impl<'a> opentelemetry::propagation::Injector for MetadataMapInjector<'a> {
    fn set(&mut self, key: &str, value: String) {

        if let Ok(metadata_key) = MetadataKey::from_bytes(key.as_bytes()) {
            let metadata_value = MetadataValue::try_from(value.as_str()).unwrap();
            self.0.insert(metadata_key, metadata_value); // Insert key and value into metadata
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize OpenTelemetry tracing and logging
    init_logging().await;
    for _i in 0..1 {
        run_method().await?;
    }
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

async fn run_method() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the gRPC server
    let mut client = ValClient::connect("http://127.0.0.1:55555").await?;

    // Create a Datapoint with a float value
    let datapoint = kuksa::val::v1::Datapoint {
        timestamp: Some(Timestamp::default()), // Default timestamp, can be set to actual time
        value: Some(Value::Float(22.0)),       // Float value as per your payload
    };

    // Create Metadata with description
    let metadata = Metadata {
        data_type: 0,                        // Specify the data type if necessary
        entry_type: 0,                       // Specify the entry type
        description: Some("16".to_string()), // Metadata description "16"
        comment: None,
        deprecation: None,
        unit: None,
        value_restriction: None,
        entry_specific: None,
    };

    // Create a DataEntry message with the path, value, and metadata
    let entry = DataEntry {
        path: "Vehicle.Speed".to_string(), // Path as per your payload
        value: Some(datapoint),            // Datapoint with float value
        actuator_target: None,
        metadata: Some(metadata), // Metadata with description
    };

    // Create an EntryUpdate message
    let entry_update = EntryUpdate {
        entry: Some(entry),
        fields: vec![Field::Value as i32, Field::MetadataDescription as i32], // Corresponding to 2 and 12 in your payload
    };

    // Create a SetRequest message
    let mut request = Request::new(SetRequest {
        updates: vec![entry_update],
    });

    // Start a span for the request
    let span = span!(Level::INFO, "publish_span");
    let _enter = span.enter(); // Span gets entered here

    // Simulate some work inside the span
    // tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Inject the current trace context into the request metadata
    let mut injector = MetadataMapInjector(request.metadata_mut());

    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&span.context(), &mut injector);
    });

    // Make the gRPC call
    let response = client.set(request).await?;

    // Print the response
    println!("Response: {:?}", response.into_inner());
    // opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

async fn init_trace() -> Result<trace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://127.0.0.1:4317"),
        )
        .with_batch_config(trace::BatchConfig::default())
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "publisher",
            )])),
        )
        .install_batch(runtime::Tokio)
}

// Initialize logging and tracing
async fn init_logging() {
    // let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Set OpenTelemetry trace propagator
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Initialize OpenTelemetry tracer
    let tracer = init_trace().await.unwrap();

    // Create telemetry layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Set up the tracing subscriber with OpenTelemetry and log formatting
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish()
        .with(telemetry); // Add telemetry layer

    // Set the subscriber as the global default for tracing
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to install global logging subscriber");

    info!("Logging initialized");
}
