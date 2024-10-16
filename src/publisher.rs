

// pub mod example {
//     tonic::include_proto!("kuksa.val.v1"); // Automatically generated from message.proto
// }

mod kuksa {
    pub mod val{
        pub mod v1{
            tonic::include_proto!("kuksa.val.v1"); // The string must match the proto package name
        }
    }
}

use kuksa::val::v1::{val_client::ValClient, SetRequest, EntryUpdate, DataEntry, Metadata};
use kuksa::val::v1::datapoint::Value;
use kuksa::val::v1::Field;
use prost_types::Timestamp;
use tonic::Request;





#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize OpenTelemetry tracing and logging
    for _i in 0..10 {
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
        timestamp: Some(Timestamp::default()),  // Default timestamp, can be set to actual time
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


    // Make the gRPC call
    let response = client.set(request).await?;

    // Print the response
    println!("Response: {:?}", response.into_inner());
    // opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

// fn main (){}