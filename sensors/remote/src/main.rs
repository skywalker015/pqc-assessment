use clap::Parser;
use serde_json::json;
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

use pqc_backend::pqc::pqc_service_client::PqcServiceClient;
use pqc_backend::pqc::DeviceResultRequest;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target host to connect to
    #[arg(short, long)]
    host: String,

    /// SSH port (default: 22)
    #[arg(short, long, default_value_t = 22)]
    port: u16,

    /// SSH username
    #[arg(short, long)]
    user: String,

    /// SSH password (optional)
    #[arg(long)]
    password: Option<String>,

    /// Path to private key (optional)
    #[arg(short = 'k', long)]
    private_key: Option<String>,

    /// Backend gRPC URL (default: http://127.0.0.1:50051)
    #[arg(long, default_value = "http://127.0.0.1:50051")]
    backend_url: String,

    /// Device ID for this host
    #[arg(long, default_value = "unknown")]
    device_id: String,
}

fn execute_command(session: &Session, cmd: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut channel = session.channel_session()?;
    channel.exec(cmd)?;
    let mut s = String::new();
    channel.read_to_string(&mut s)?;
    channel.wait_close()?;
    Ok(s.trim().to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Connecting to {}:{}...", args.host, args.port);
    let tcp = TcpStream::connect(format!("{}:{}", args.host, args.port))?;
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;

    if let Some(pwd) = &args.password {
        session.userauth_password(&args.user, pwd)?;
    } else if let Some(key_path) = &args.private_key {
        session.userauth_pubkey_file(&args.user, None, std::path::Path::new(key_path), None)?;
    } else {
        session.userauth_agent(&args.user)?;
    }

    if !session.authenticated() {
        return Err("Authentication failed".into());
    }
    println!("Authenticated successfully!");

    // Run some config checks
    let openssl_version = execute_command(&session, "openssl version").unwrap_or_else(|_| "unknown".to_string());
    let ssh_version = execute_command(&session, "ssh -V 2>&1").unwrap_or_else(|_| "unknown".to_string());
    
    // Check sshd_config for PQC settings like KexAlgorithms
    let sshd_config = execute_command(&session, "cat /etc/ssh/sshd_config | grep -i KexAlgorithms").unwrap_or_else(|_| "".to_string());

    let payload = json!({
        "host": args.host,
        "openssl_version": openssl_version,
        "ssh_version": ssh_version,
        "sshd_kex": sshd_config,
    });

    println!("Collected data: {}", payload);

    let mut client = PqcServiceClient::connect(args.backend_url.clone()).await?;

    let request = DeviceResultRequest {
        sensor_id: "remote-agent-01".to_string(), // In reality, maybe dynamically generate or configure
        device_id: args.device_id,
        result_type: "remote-config".to_string(),
        payload_json: payload.to_string(),
        collected_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    };

    let response = client.submit_device_result(request).await?;
    println!("Server response: {:?}", response.into_inner());

    Ok(())
}
