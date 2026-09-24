use std::time::{SystemTime, UNIX_EPOCH};

use pqc_backend::pqc::pqc_service_client::PqcServiceClient;
use pqc_backend::pqc::DeviceResultRequest;

#[tokio::main]
async fn main() {
    let mut client = PqcServiceClient::connect("http://127.0.0.1:50051")
        .await
        .expect("failed to connect to backend");

    let request = DeviceResultRequest {
        sensor_id: "device-agent-01".to_string(),
        device_id: "device-01".to_string(),
        result_type: "config-check".to_string(),
        payload_json: r#"{"openssl":"OpenSSL 3.0.2","openssh":"OpenSSH_9.6p1"}"#.to_string(),
        collected_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    };

    let token = std::env::var("PQC_SENSOR_API_TOKEN")
        .expect("PQC_SENSOR_API_TOKEN must be set");
    let mut request = tonic::Request::new(request);
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {token}").parse().expect("invalid auth metadata"),
    );

    let response = client
        .submit_device_result(request)
        .await
        .expect("submit_device_result failed");

    println!("server response: {:?}", response.into_inner());
}
