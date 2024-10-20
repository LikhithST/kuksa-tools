mod kuksa {
    pub mod val {
        pub mod v1 {
            tonic::include_proto!("kuksa.val.v1"); // The string must match the proto package name
        }
    }
}

use std::time::SystemTime;

use kuksa::val::v1::{val_client::ValClient, SetRequest, EntryUpdate, DataEntry, Metadata};
use kuksa::val::v1::datapoint::Value;
use kuksa::val::v1::Field;
use prost_types::Timestamp;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize OpenTelemetry tracing and logging (if needed)
    
    // Connect to the gRPC server once
    let mut client = ValClient::connect("http://127.0.0.1:55555").await?;

    // Call the method multiple times, reusing the same connection
    for _i in 0..1000 {
        run_method(&mut client).await?;
    }

    // Shutdown tracing after all calls are done
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

async fn run_method(client: &mut ValClient<tonic::transport::Channel>) -> Result<(), Box<dyn std::error::Error>> {

    // Create a Datapoint with a float value
    let datapoint = kuksa::val::v1::Datapoint {
        timestamp: Some(SystemTime::now().into()),  // Default timestamp, can be set to actual time
        value: Some(Value::Float(22.0)),        // Float value as per your payload
    };

    // Create Metadata with description
    let metadata = Metadata {
        data_type: 0,               // Specify the data type if necessary
        entry_type: 0,              // Specify the entry type
        description: Some("16".to_string()), // Metadata description "16"
        comment: None,
        deprecation: None,
        unit: None,
        value_restriction: None,
        entry_specific: None,
    };

    // Create a DataEntry message with the path, value, and metadata
    let entry = DataEntry {
        path: "Vehicle.Speed".to_string(),  // Path as per your payload
        value: Some(datapoint),             // Datapoint with float value
        actuator_target: None,
        metadata: Some(metadata),           // Metadata with description
    };

    // Create an EntryUpdate message
    let entry_update = EntryUpdate {
        entry: Some(entry),
        fields: vec![Field::Value as i32, Field::MetadataDescription as i32], // Corresponding to 2 and 12 in your payload
    };

    // Create a SetRequest message
    let request = Request::new(SetRequest {
        updates: vec![entry_update],
    });

    // Make the gRPC call using the same client connection
    let response = client.set(request).await?;

    // Print the response
    //println!("Response: {:?}", response.into_inner());

    Ok(())
}
