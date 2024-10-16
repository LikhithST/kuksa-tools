use kuksa::val::v1::{val_client::ValClient, Field, SubscribeEntry, SubscribeRequest, View};
use tokio_stream::StreamExt;
use tonic::Request;

mod kuksa {
    pub mod val {
        pub mod v1 {
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
                println!("Received Update: {:?}", response);
            }
            Err(e) => {
                eprintln!("Error receiving update: {}", e);
            }
        }
    }

    Ok(())
}
