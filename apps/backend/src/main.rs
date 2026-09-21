use std::net::SocketAddr;

use pqc_backend::{config::BackendConfig, db::Database, grpc::serve_grpc};

#[tokio::main]
async fn main() {
    let config = BackendConfig::default();
    let database = Database::new(&config.database_url);
    database.init().expect("failed to initialize database");

    let grpc_addr = config.grpc_listen_addr.clone();

    tokio::spawn(async move {
        if let Err(err) = serve_grpc(&grpc_addr).await {
            eprintln!("gRPC server failed: {err}");
        }
    });

    let http_addr: SocketAddr = config.http_listen_addr.parse().unwrap();
    let app = pqc_backend::api::router();

    println!("HTTP server listening on {}", http_addr);

    let listener = tokio::net::TcpListener::bind(http_addr)
        .await
        .expect("failed to bind HTTP listener");

    axum::serve(listener, app)
        .await
        .expect("failed to serve HTTP");
}
