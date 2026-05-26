use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use tracing::info;

use crate::{
    rpc::SorobanRpcClient,
    types::{ErrorResponse, FeeEstimate, SimulateRequest, SimulateResponse, SimulationDetail},
};

/// GET /health
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

/// POST /simulate
///
/// Calls the Soroban RPC `simulateTransaction` endpoint to get real fee
/// estimates and simulation results. Falls back to heuristic estimates if
/// the RPC is unavailable.
pub async fn simulate(
    State(rpc): State<SorobanRpcClient>,
    Json(req): Json<SimulateRequest>,
) -> Result<Json<SimulateResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate target: Stellar contract IDs are 56-char base32 strings starting with C
    if req.target.is_empty() || req.function.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "target and function are required".to_string(),
            }),
        ));
    }
    if req.target.len() != 56 || !req.target.starts_with('C') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "target must be a 56-character Stellar contract ID starting with C"
                    .to_string(),
            }),
        ));
    }

    info!(target = %req.target, function = %req.function, "simulating transaction");

    let breakdown = rpc
        .simulate(&req.target, &req.function, req.amount as i64, req.network_load_bps)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e.to_string() }),
            )
        })?;

    let response = SimulateResponse {
        success: breakdown.would_succeed,
        estimated_fees: FeeEstimate {
            base_fee: breakdown.base_fee,
            resource_fee: breakdown.resource_fee,
            total_fee: breakdown.total_fee,
            surge_multiplier: breakdown.surge_multiplier,
            high_load: breakdown.high_load,
        },
        simulation: SimulationDetail {
            target: req.target,
            function: req.function,
            would_succeed: breakdown.would_succeed,
        },
        message: if breakdown.would_succeed {
            "Simulation successful".to_string()
        } else {
            "Simulation indicates transaction would fail".to_string()
        },
    };

    Ok(Json(response))
}
