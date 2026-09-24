#[derive(Debug, Clone)]
pub struct BackendConfig {
    pub database_url: String,
    pub http_listen_addr: String,
    pub grpc_listen_addr: String,
    pub sensor_api_token: Option<String>,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            database_url: "./data/pqc_assessment.db".to_string(),
            http_listen_addr: "0.0.0.0:3000".to_string(),
            grpc_listen_addr: "0.0.0.0:50051".to_string(),
            sensor_api_token: std::env::var("PQC_SENSOR_API_TOKEN").ok(),
        }
    }
}
