pub mod api;
pub mod config;
pub mod db;
pub mod grpc;
pub mod pqc {
    tonic::include_proto!("pqc");
}
pub mod services;

pub fn backend_name() -> &'static str {
    "pqc-backend"
}
