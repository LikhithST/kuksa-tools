use kuksa::val::v1::{val_client::ValClient, SubscribeRequest, SubscribeEntry, View, Field, EntryUpdate, DataEntry};
use tonic::{Request, metadata::MetadataMap, metadata::MetadataValue, metadata::MetadataKey};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tokio_stream::StreamExt; 
use tracing::{span, Level, info};
use opentelemetry::trace::TraceError;
use opentelemetry::sdk::{trace, Resource, propagation::TraceContextPropagator};
use opentelemetry::{global, KeyValue, runtime};
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::layer::SubscriberExt;
use tonic::metadata::KeyAndValueRef;
mod kuksa {
    pub mod val{
        pub mod v1{
            tonic::include_proto!("kuksa.val.v1"); // The string must match the proto package name
        }
    }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the gRPC client
    let mut client = ValClient::connect("http://127.0.0.1:55555").await?;
    init_logging().await;

    // Create the SubscribeEntry as per your payload
    let subscribe_entry = SubscribeEntry {
        path: "Vehicle.Speed".to_string(),
        view: View::All.into(), // Use the corresponding enum for VIEW_ALL
        fields: vec![Field::Value.into()], // Convert "FIELD_VALUE" to the corresponding enum
    };

    // Create the SubscribeRequest
    let request = Request::new(SubscribeRequest {
        entries: vec![subscribe_entry],
    });


   

    // Send the Subscribe request and await the response stream
    let mut response_stream = client.subscribe(request).await?.into_inner();

    // Handle the streamed responses
    println!("Subscribed to changes...");
    while let Some(subscribe_response) = response_stream.next().await {
        match subscribe_response {
            Ok(response) => {
                 // Start a span for the request

                let span = span!(Level::INFO, "subscribe_span");
                let _enter = span.enter(); // Span gets entered here

                println!("Received Update: {:?}", response);
                let description:Option<String> = match response.updates.get(0) {
                    // If there's an EntryUpdate
                    Some(EntryUpdate { entry: Some(DataEntry { metadata: Some(kuksa::val::v1::Metadata { description: Some(description), .. }), .. }), .. }) => {
                        // Return the description
                        Some(description.clone()) // Clone the description since it's a reference
                    }
                    // Default case: return None if the description doesn't exist
                    _ => None,
                };


                    // Convert to MetadataMap
    match string_to_metadata_map(&description.as_deref().unwrap_or("")) {
        Ok(metadata) => {
            let cx = global::get_text_map_propagator(|propagator| {
                propagator.extract(&MetadataMapExtractor(&metadata))
            });
            tracing::Span::current().set_parent(cx);
        }
        Err(e) => {
            eprintln!("Error converting string to MetadataMap: {:?}", e);
        }
    }

            }
            Err(e) => {
                eprintln!("Error receiving update: {}", e);
            }
        }
    }
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

// Metadata extractor for gRPC
struct MetadataMapExtractor<'a>(&'a tonic::metadata::MetadataMap);

impl<'a> opentelemetry::propagation::Extractor for MetadataMapExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|val| val.to_str().ok())
    }

     /// Collect all the keys from the HeaderMap.
     fn keys(&self) -> Vec<&str> {
        self.0.iter()
            .filter_map(|kv| {
                if let KeyAndValueRef::Ascii(key, _) = kv {
                    Some(key.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}

fn string_to_metadata_map(input: &str) -> Result<MetadataMap, Box<dyn std::error::Error>> {
    let mut metadata = MetadataMap::new();

    for line in input.lines() {
        // Split the line at the first colon ':', to separate key and value
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue; // Skip lines that don't match the format
        }

        let key = parts[0].trim();   // Trim whitespace around the key
        let value = parts[1].trim(); // Trim whitespace around the value

        // Try to create MetadataKey and MetadataValue
        if let Ok(metadata_key) = MetadataKey::from_bytes(key.as_bytes()) {
            let metadata_value = MetadataValue::try_from(value)?;
            metadata.insert(metadata_key, metadata_value);
        }
    }


    Ok(metadata)
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
                "subscribe",
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