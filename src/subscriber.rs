use std::time::SystemTime;

use kuksa::val::v1::{val_client::ValClient, SubscribeRequest, SubscribeResponse, SubscribeEntry, View, Field};
use tonic::Request;
use tokio_stream::StreamExt;
use prost_types::Timestamp;
use tracing_subscriber::fmt::time;

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
                let now: SystemTime = SystemTime::now();
                //println!("Received Update: {:?}", response);
                // convert response to SubscribeResponse
                let res: SubscribeResponse = response;
                if res.updates[0].entry.is_none() {
                    continue;
                }
                let entry: kuksa::val::v1::DataEntry = res.updates[0].entry.clone().unwrap();
                //println!("datapoint: {:?}", entry);
                if entry.value.is_none() {
                    continue;
                }
                let datapoint = entry.value.unwrap();
                let timestamp = datapoint.timestamp.unwrap();
                // create SystemTime from Timestamp
                let timestamp: SystemTime = SystemTime::UNIX_EPOCH + std::time::Duration::new(timestamp.seconds as u64, timestamp.nanos as u32);
                println!("{:?}", now.duration_since(timestamp).unwrap().as_micros());
            }
            Err(e) => {
                eprintln!("Error receiving update: {}", e);
            }
        }
    }

    Ok(())
}
