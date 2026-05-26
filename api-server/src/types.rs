use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateRequest {
    /// Target contract address (56-char Stellar contract ID starting with C)
    pub target: String,
    /// Function name to invoke
    pub function: String,
    /// Transaction amount in stroops (used for fee estimation)
    #[serde(default = "default_amount")]
    pub amount: i64,
    /// Fee rate in basis points (default 30 = 0.30%)
    #[serde(default = "default_fee_bps")]
    pub fee_bps: u32,
    /// Network load in basis points for surge pricing (0–10000)
    #[serde(default)]
    pub network_load_bps: u32,
}

fn default_amount() -> i64 { 1_000_000 }
fn default_fee_bps() -> u32 { 30 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateResponse {
    pub success: bool,
    pub estimated_fees: FeeEstimate,
    pub simulation: SimulationDetail,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimate {
    pub base_fee: i64,
    pub resource_fee: i64,
    pub total_fee: i64,
    pub surge_multiplier: u32,
    pub high_load: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationDetail {
    pub target: String,
    pub function: String,
    pub would_succeed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
