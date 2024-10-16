# kuksa-tools

## Kuksa Databroker with otel
env OTEL_BSP_MAX_QUEUE_SIZE=8192 cargo run --bin databroker --features otel -- --address 127.0.0.1 --metadata ./data/vss-core/vss_release_4.0.json --insecure

## Kuksa Databroker 
cargo run --bin databroker -- --address 127.0.0.1 --metadata ./data/vss-core/vss_release_4.0.json --insecure

## Kuksa publisher
cargo run --bin publisher

## Kuksa publisher with otel
cargo run --bin publisher-otel

## kuksa subscriber
cargo run --bin subscriber