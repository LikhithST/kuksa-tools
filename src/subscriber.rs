use kuksa::val::v1::{val_client::ValClient, SubscribeRequest, SubscribeEntry, View, Field};
use tonic::{Request, metadata::MetadataMap, metadata::MetadataValue, metadata::MetadataKey};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tokio_stream::StreamExt; 
use tracing::{span, Level};
use rand::Rng;
mod kuksa {
    pub mod val{
        pub mod v1{
            tonic::include_proto!("kuksa.val.v1"); // The string must match the proto package name
        }
    }
}


struct MetadataMapInjector<'a>(&'a mut MetadataMap);

impl<'a> opentelemetry::propagation::Injector for MetadataMapInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(metadata_key) = MetadataKey::from_bytes(key.as_bytes()) {
            let metadata_value = MetadataValue::try_from(value.as_str()).unwrap();
            self.0.insert(metadata_key, metadata_value);  // Insert key and value into metadata
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the gRPC client
    let mut client = ValClient::connect("http://127.0.0.1:55555").await?;

    // Create the SubscribeEntry as per your payload
    let subscribe_entry = SubscribeEntry {
        path: "Vehicle.Speed".to_string(),
        view: View::All.into(), // Use the corresponding enum for VIEW_ALL
        fields: vec![Field::Value.into()], // Convert "FIELD_VALUE" to the corresponding enum
    };

    // Create the SubscribeRequest
    let mut request = Request::new(SubscribeRequest {
        entries: vec![subscribe_entry],
    });

    let random_number: u32 = rand::thread_rng().gen_range(1000..10000);
    let random_token = format!("token-{}", random_number);
    let metadata_value = MetadataValue::try_from(random_token.as_str())?;
    request.metadata_mut().insert("authorization", metadata_value);

    // Start a span for the request
    let span = span!(Level::INFO, "server_request");
    let _enter = span.enter(); // Span gets entered here

    let mut injector = MetadataMapInjector(request.metadata_mut());

    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&span.context(), &mut injector);
    });

    // Send the Subscribe request and await the response stream
    let mut response_stream = client.subscribe(request).await?.into_inner();

    // Handle the streamed responses
    println!("Subscribed to changes...");
    while let Some(subscribe_response) = response_stream.next().await {
        match subscribe_response {
            Ok(response) => {
                println!("Received Update: {:?}", response);
            }
            Err(e) => {
                eprintln!("Error receiving update: {}", e);
            }
        }
    }

    Ok(())
}
