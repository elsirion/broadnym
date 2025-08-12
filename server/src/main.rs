use anyhow::Result;
use axum::{response::Html, routing::get, Router};
use clap::Parser;
use common::TransactionRequest;
use directories::ProjectDirs;
use futures::StreamExt;
use nym_sdk::mixnet::{MixnetClientBuilder, StoragePaths};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "3000")]
    port: u16,
    
    /// Custom data directory for storing Nym client configuration
    #[arg(long)]
    data_dir: Option<PathBuf>,
}

struct AppState {
    nym_address: Arc<RwLock<Option<String>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let state = Arc::new(AppState {
        nym_address: Arc::new(RwLock::new(None)),
    });

    let state_clone = state.clone();
    let data_dir = args.data_dir.clone();
    tokio::spawn(async move {
        if let Err(e) = run_nym_service(state_clone, data_dir).await {
            error!("Nym service error: {}", e);
        }
    });

    let app = Router::new()
        .route("/", get(index_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port))
        .await?;
    
    info!("Web server listening on http://0.0.0.0:{}", args.port);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Html<String> {
    let nym_address = state.nym_address.read().await;
    
    let html = match &*nym_address {
        Some(addr) => format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Bitcoin Transaction Broadcaster</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .address {{ 
            background: #f0f0f0; 
            padding: 20px; 
            border-radius: 8px;
            word-break: break-all;
            font-family: monospace;
        }}
    </style>
</head>
<body>
    <h1>Bitcoin Transaction Broadcaster</h1>
    <h2>Nym Service Address:</h2>
    <div class="address">{}</div>
    <p>Send transaction requests to this address via the Nym mixnet.</p>
</body>
</html>"#,
            addr
        ),
        None => r#"<!DOCTYPE html>
<html>
<head>
    <title>Bitcoin Transaction Broadcaster</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
    </style>
</head>
<body>
    <h1>Bitcoin Transaction Broadcaster</h1>
    <p>Nym service is starting up, please refresh in a moment...</p>
</body>
</html>"#.to_string(),
    };
    
    Html(html)
}

async fn run_nym_service(state: Arc<AppState>, custom_data_dir: Option<PathBuf>) -> Result<()> {
    info!("Starting Nym mixnet client...");
    
    // Determine storage directory
    let storage_dir = match custom_data_dir {
        Some(dir) => dir,
        None => {
            let project_dirs = ProjectDirs::from("com", "broadnym", "server")
                .ok_or_else(|| anyhow::anyhow!("Could not determine project directories"))?;
            project_dirs.data_dir().join("nym-client")
        }
    };
    
    // Create directory if it doesn't exist
    tokio::fs::create_dir_all(&storage_dir).await?;
    info!("Using storage directory: {:?}", storage_dir);
    
    // Create storage paths for persistent client
    let storage_paths = StoragePaths::new_from_dir(&storage_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create storage paths: {}", e))?;
    
    let mut client = MixnetClientBuilder::new_with_default_storage(storage_paths)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create client with storage: {}", e))?
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build mixnet client: {}", e))?
        .connect_to_mixnet()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to mixnet: {}", e))?;
    
    let address = client.nym_address().to_string();
    info!("Nym service listening on: {}", address);
    
    {
        let mut nym_addr = state.nym_address.write().await;
        *nym_addr = Some(address.to_string());
    }
    
    loop {
        match client.next().await {
            Some(packet) => {
                let message = packet.message;
                match handle_request(message).await {
                    Ok(_response) => {
                        info!("Processed transaction request successfully");
                    },
                    Err(e) => {
                        error!("Request handling error: {}", e);
                    }
                }
            }
            None => {
                error!("Client connection closed");
                break;
            }
        }
    }
    
    Ok(())
}

async fn handle_request(request: Vec<u8>) -> Result<Vec<u8>> {
    let tx_request: TransactionRequest = bincode::deserialize(&request)?;
    info!("Received transaction request for {:?}", tx_request.network);
    
    let client = reqwest::Client::new();
    let url = format!("{}/tx", tx_request.network.mempool_api_url());
    
    let response = client
        .post(&url)
        .header("Content-Type", "text/plain")
        .body(tx_request.tx_hex)
        .send()
        .await?;
    
    let status = response.status();
    let body = response.text().await?;
    
    if status.is_success() {
        info!("Transaction submitted successfully: {}", body);
        Ok(format!("Success: {}", body).into_bytes())
    } else {
        error!("Transaction submission failed: {} - {}", status, body);
        Err(anyhow::anyhow!("Submission failed: {} - {}", status, body))
    }
}
