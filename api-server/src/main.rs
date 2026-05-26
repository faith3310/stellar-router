mod handlers;
mod rpc;
mod types;

#[cfg(test)]
mod tests;

use axum::{routing::{get, post}, Router};
use rpc::SorobanRpcClient;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let rpc_url = env::var("SOROBAN_RPC_URL")
        .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string());

    let listen_addr = env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    let rpc = SorobanRpcClient::new(rpc_url);

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/simulate", post(handlers::simulate))
        .with_state(rpc);

    tracing::info!("listening on {}", listen_addr);
    let listener = tokio::net::TcpListener::bind(&listen_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
