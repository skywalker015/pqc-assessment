use std::time::{SystemTime, UNIX_EPOCH};

use pqc_backend::pqc::pqc_service_client::PqcServiceClient;
use pqc_backend::pqc::ObservationRequest;

#[tokio::main]
async fn main() {
    let mut client = PqcServiceClient::connect("http://127.0.0.1:50051")
        .await
        .expect("failed to connect to backend");

    let request = ObservationRequest {
        sensor_id: "network-sensor-01".to_string(),
        device_id: "device-01".to_string(),
        asset_id: "asset-01".to_string(),
        observation_type: "tls".to_string(),
        payload_json: r#"{"protocol":"tls","tls_version":"1.3"}"#.to_string(),
        observed_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    };

    let response = client
        .submit_observation(request)
        .await
        .expect("submit_observation failed");

    println!("server response: {:?}", response.into_inner());
}
