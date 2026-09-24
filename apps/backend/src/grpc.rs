use subtle::ConstantTimeEq;
use tonic::{Request, Response, Status};

use crate::{
    config::BackendConfig,
    db::{Database, ObservationRepository, SensorRepository, SensorTokenRepository},
    pqc::{
        pqc_service_server::{PqcService, PqcServiceServer},
        DeviceResultRequest, HealthRequest, HealthResponse, ObservationRequest, SubmitResponse,
    },
};

pub struct PqcBackendService {
    sensor_api_token: Option<String>,
}

impl PqcBackendService {
    pub fn new(sensor_api_token: Option<String>) -> Self {
        Self { sensor_api_token }
    }
}

#[tonic::async_trait]
impl PqcService for PqcBackendService {
    async fn submit_observation(
        &self,
        request: Request<ObservationRequest>,
    ) -> Result<Response<SubmitResponse>, Status> {
        let authorization = authorization(&request)?;
        let req = request.get_ref();
        let db = Database::new(&BackendConfig::default().database_url);
        authorize_sensor(
            &self.sensor_api_token,
            &authorization,
            &req.sensor_id,
            &db,
        )?;
        let sensor_repository = SensorRepository::new(&db).map_err(internal_error)?;
        if !sensor_repository
            .heartbeat(&req.sensor_id, &req.observed_at)
            .map_err(internal_error)?
        {
            return Err(Status::permission_denied("sensor is not registered"));
        }
        let req = request.into_inner();
        let repo = ObservationRepository::new(&db).map_err(internal_error)?;

        repo.save_observation(
            &req.sensor_id,
            Some(&req.device_id),
            Some(&req.asset_id),
            &req.observation_type,
            &req.payload_json,
            &req.observed_at,
        )
        .map_err(|err| Status::internal(err.to_string()))?;

        let response = SubmitResponse {
            accepted: true,
            message: format!(
                "Observation accepted from sensor {} for device {}",
                req.sensor_id, req.device_id
            ),
        };

        Ok(Response::new(response))
    }

    async fn submit_device_result(
        &self,
        request: Request<DeviceResultRequest>,
    ) -> Result<Response<SubmitResponse>, Status> {
        let authorization = authorization(&request)?;
        let req = request.get_ref();
        let db = Database::new(&BackendConfig::default().database_url);
        authorize_sensor(
            &self.sensor_api_token,
            &authorization,
            &req.sensor_id,
            &db,
        )?;
        let sensor_repository = SensorRepository::new(&db).map_err(internal_error)?;
        if !sensor_repository
            .heartbeat(&req.sensor_id, &req.collected_at)
            .map_err(internal_error)?
        {
            return Err(Status::permission_denied("sensor is not registered"));
        }
        let req = request.into_inner();
        let repo = ObservationRepository::new(&db).map_err(internal_error)?;

        repo.save_observation(
            &req.sensor_id,
            Some(&req.device_id),
            None,
            &req.result_type,
            &req.payload_json,
            &req.collected_at,
        )
        .map_err(|err| Status::internal(err.to_string()))?;

        let response = SubmitResponse {
            accepted: true,
            message: format!(
                "Device result accepted from sensor {} for device {}",
                req.sensor_id, req.device_id
            ),
        };

        Ok(Response::new(response))
    }

    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            status: "ok".to_string(),
            service: "pqc-backend".to_string(),
        }))
    }
}

pub async fn serve_grpc(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse()?;
    let service = PqcBackendService::new(BackendConfig::default().sensor_api_token);

    println!("gRPC server listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(PqcServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

fn authorization(request: &Request<impl prost::Message>) -> Result<&str, Status> {
    request
        .metadata()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Status::unauthenticated("missing bearer token"))
}

fn authorize_sensor(
    bootstrap_token: &Option<String>,
    provided_token: &str,
    sensor_id: &str,
    database: &Database,
) -> Result<(), Status> {
    let bootstrap_matches = bootstrap_token.as_deref().is_some_and(|expected| {
        let matches: bool = expected.as_bytes().ct_eq(provided_token.as_bytes()).into();
        matches
    });
    if bootstrap_matches {
        return Ok(());
    }

    let repository = SensorTokenRepository::new(database).map_err(internal_error)?;
    let valid = repository
        .is_valid(sensor_id, provided_token, &chrono::Utc::now().to_rfc3339())
        .map_err(internal_error)?;
    if valid {
        Ok(())
    } else {
        Err(Status::unauthenticated("invalid or expired sensor token"))
    }
}

fn internal_error(error: rusqlite::Error) -> Status {
    Status::internal(error.to_string())
}
