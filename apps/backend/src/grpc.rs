use tonic::{Request, Response, Status};

use crate::{
    config::BackendConfig,
    db::{Database, ObservationRepository},
    pqc::{
        pqc_service_server::{PqcService, PqcServiceServer},
        DeviceResultRequest, HealthRequest, HealthResponse, ObservationRequest, SubmitResponse,
    },
};

#[derive(Default)]
pub struct PqcBackendService;

#[tonic::async_trait]
impl PqcService for PqcBackendService {
    async fn submit_observation(
        &self,
        request: Request<ObservationRequest>,
    ) -> Result<Response<SubmitResponse>, Status> {
        let req = request.into_inner();
        let db = Database::new(&BackendConfig::default().database_url);
        let repo = ObservationRepository::new(&db).map_err(|err| Status::internal(err.to_string()))?;

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
        let req = request.into_inner();
        let db = Database::new(&BackendConfig::default().database_url);
        let repo = ObservationRepository::new(&db).map_err(|err| Status::internal(err.to_string()))?;

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
    let service = PqcBackendService::default();

    println!("gRPC server listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(PqcServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
