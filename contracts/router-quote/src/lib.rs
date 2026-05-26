#![no_std]

//! # router-quote
//!
//! Preview transaction results before execution.
//! Returns expected output amount, fees, exchange rate, and route details
//! without executing the transaction.
//!
//! ## Exchange Rate
//!
//! Exchange rates are calculated as a fixed-point integer with configurable
//! decimal precision. For example, with `precision = 6`:
//!
//!   exchange_rate = (amount_out * 10^precision) / amount_in
//!
//! A rate of `1_050_000` with precision 6 means 1.050000 token_out per token_in.
//! This avoids floating point entirely and is safe for on-chain use.

extern crate alloc;
use alloc::string::ToString;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, String, Symbol, Vec,
};

// ── Types ─────────────────────────────────────────────────────────────────────

/// Response containing quote details.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct QuoteResponse {
    /// The expected output amount (token_out units).
    pub amount_out: i128,
    /// The fee amount deducted (in token_in units).
    pub fee_amount: i128,
    /// The route name that was used.
    pub route_name: String,
    /// The target contract address.
    pub target: Address,
    /// Minimum output amount after slippage tolerance.
    pub min_amount_out: i128,
    /// Exchange rate as a fixed-point integer.
    /// Value = (amount_out * 10^precision) / amount_in.
    /// Use `precision` field to interpret.
    pub exchange_rate: i128,
    /// Number of decimal places in `exchange_rate`.
    pub precision: u32,
    /// Price impact in basis points (negative = adverse).
    pub price_impact_bps: i32,
}

/// Fee estimate breakdown.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct FeeEstimateResponse {
    /// Protocol fee in token_in base units.
    pub protocol_fee: i128,
    /// Network fee in stroops.
    pub network_fee: i128,
    /// Total fee (protocol + network).
    pub total_fee: i128,
    /// Whether surge pricing was applied.
    pub surge_pricing: bool,
    /// Effective fee rate in basis points after surge adjustment.
    pub effective_fee_bps: u32,
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum QuoteError {
    InvalidAmount = 1,
    RouteNotFound = 2,
    QuoteFailed = 3,
    InvalidPrecision = 4,
    InvalidSlippage = 5,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct RouterQuote;

#[contractimpl]
impl RouterQuote {
    /// Get a quote from a liquidity plugin contract.
    ///
    /// Calls `get_quote(token_in, token_out, amount_in) -> i128` on `target`
    /// and returns a full [`QuoteResponse`] including a properly calculated
    /// exchange rate.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `target` - The liquidity plugin contract address.
    /// * `route_name` - Human-readable name for the route.
    /// * `token_in` - Token being sold.
    /// * `token_out` - Token being bought.
    /// * `amount_in` - Amount of token_in to swap (must be > 0).
    /// * `fee_bps` - Protocol fee in basis points (e.g. 30 = 0.30%).
    /// * `slippage_bps` - Slippage tolerance in basis points (e.g. 50 = 0.50%).
    /// * `precision` - Decimal places for the exchange rate (1–18, typically 6).
    ///
    /// # Returns
    /// A [`QuoteResponse`] with `exchange_rate` = `(amount_out * 10^precision) / amount_in`.
    ///
    /// # Errors
    /// * [`QuoteError::InvalidAmount`] — if `amount_in` ≤ 0.
    /// * [`QuoteError::InvalidPrecision`] — if `precision` is 0 or > 18.
    /// * [`QuoteError::InvalidSlippage`] — if `slippage_bps` > 10000.
    /// * [`QuoteError::QuoteFailed`] — if the plugin call fails.
    pub fn get_quote(
        env: Env,
        target: Address,
        route_name: String,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        fee_bps: u32,
        slippage_bps: u32,
        precision: u32,
    ) -> Result<QuoteResponse, QuoteError> {
        if amount_in <= 0 {
            return Err(QuoteError::InvalidAmount);
        }
        if precision == 0 || precision > 18 {
            return Err(QuoteError::InvalidPrecision);
        }
        if slippage_bps > 10_000 {
            return Err(QuoteError::InvalidSlippage);
        }

        // Call the plugin's get_quote function
        let function = Symbol::new(&env, "get_quote");
        let mut args = Vec::new(&env);
        args.push_back(token_in.into());
        args.push_back(token_out.into());
        args.push_back(amount_in.into());

        let amount_out: i128 = env
            .try_invoke_contract::<i128, i128>(&target, &function, args)
            .map_err(|_| QuoteError::QuoteFailed)?
            .map_err(|_| QuoteError::QuoteFailed)?;

        // Protocol fee: amount_in * fee_bps / 10_000
        let fee_amount = amount_in * fee_bps as i128 / 10_000;

        // Slippage: min_amount_out = amount_out * (10_000 - slippage_bps) / 10_000
        let min_amount_out = amount_out * (10_000 - slippage_bps as i128) / 10_000;

        // Exchange rate as fixed-point: (amount_out * 10^precision) / amount_in
        // Uses i128 arithmetic — safe for precision ≤ 18 and typical token amounts.
        let scale = Self::pow10(precision);
        let exchange_rate = (amount_out * scale) / amount_in;

        // Price impact: simplified as (amount_out - amount_in) * 10_000 / amount_in
        // Negative means the user receives less than they put in (adverse).
        let price_impact_bps = ((amount_out - amount_in) * 10_000 / amount_in) as i32;

        env.events().publish(
            (Symbol::new(&env, "quote_generated"),),
            (&target, amount_in, amount_out, exchange_rate),
        );

        Ok(QuoteResponse {
            amount_out,
            fee_amount,
            route_name,
            target,
            min_amount_out,
            exchange_rate,
            precision,
            price_impact_bps,
        })
    }

    /// Estimate fees for a transaction.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `amount` - Transaction amount in token base units (must be > 0).
    /// * `fee_bps` - Route fee rate in basis points.
    /// * `network_load_bps` - Current network utilization (0–10000).
    ///   Values ≥ 8000 trigger 2× surge pricing on the network fee.
    ///
    /// # Errors
    /// * [`QuoteError::InvalidAmount`] — if `amount` ≤ 0.
    pub fn estimate_fee(
        env: Env,
        amount: i128,
        fee_bps: u32,
        network_load_bps: u32,
    ) -> Result<FeeEstimateResponse, QuoteError> {
        if amount <= 0 {
            return Err(QuoteError::InvalidAmount);
        }

        let protocol_fee = amount * fee_bps as i128 / 10_000;
        let base_network_fee: i128 = 100; // 100 stroops minimum

        let (network_fee, surge_pricing, effective_fee_bps) = if network_load_bps >= 8_000 {
            (base_network_fee * 2, true, fee_bps * 2)
        } else {
            (base_network_fee, false, fee_bps)
        };

        Ok(FeeEstimateResponse {
            protocol_fee,
            network_fee,
            total_fee: protocol_fee + network_fee,
            surge_pricing,
            effective_fee_bps,
        })
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Returns 10^exp as i128. Safe for exp ≤ 18.
    fn pow10(exp: u32) -> i128 {
        let mut result: i128 = 1;
        let mut i = 0u32;
        while i < exp {
            result *= 10;
            i += 1;
        }
        result
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, Symbol};

    // A minimal mock liquidity plugin that returns amount_in * 2
    #[soroban_sdk::contract]
    pub struct MockPlugin;

    #[soroban_sdk::contractimpl]
    impl MockPlugin {
        pub fn get_quote(_env: Env, _token_in: Address, _token_out: Address, amount_in: i128) -> i128 {
            amount_in * 2
        }
    }

    fn setup() -> (Env, RouterQuoteClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, RouterQuote);
        let client = RouterQuoteClient::new(&env, &contract_id);
        let plugin_id = env.register_contract(None, MockPlugin);
        (env, client, plugin_id)
    }

    #[test]
    fn test_exchange_rate_calculated_correctly() {
        let (env, client, plugin) = setup();
        let token_in = Address::generate(&env);
        let token_out = Address::generate(&env);
        let route = String::from_str(&env, "mock/pool");

        // amount_in = 1_000_000, plugin returns 2_000_000
        // exchange_rate = (2_000_000 * 10^6) / 1_000_000 = 2_000_000
        let resp = client.get_quote(
            &plugin, &route, &token_in, &token_out,
            &1_000_000, &0, &0, &6,
        );
        assert_eq!(resp.amount_out, 2_000_000);
        assert_eq!(resp.exchange_rate, 2_000_000); // 2.000000 with precision 6
        assert_eq!(resp.precision, 6);
    }

    #[test]
    fn test_exchange_rate_precision_2() {
        let (env, client, plugin) = setup();
        let token_in = Address::generate(&env);
        let token_out = Address::generate(&env);
        let route = String::from_str(&env, "mock/pool");

        // amount_in = 100, plugin returns 200
        // exchange_rate = (200 * 10^2) / 100 = 200 → 2.00 with precision 2
        let resp = client.get_quote(
            &plugin, &route, &token_in, &token_out,
            &100, &0, &0, &2,
        );
        assert_eq!(resp.exchange_rate, 200);
        assert_eq!(resp.precision, 2);
    }

    #[test]
    fn test_fee_deducted_correctly() {
        let (env, client, plugin) = setup();
        let token_in = Address::generate(&env);
        let token_out = Address::generate(&env);
        let route = String::from_str(&env, "mock/pool");

        // fee_bps = 30 (0.30%), amount_in = 1_000_000
        // fee = 1_000_000 * 30 / 10_000 = 3_000
        let resp = client.get_quote(
            &plugin, &route, &token_in, &token_out,
            &1_000_000, &30, &0, &6,
        );
        assert_eq!(resp.fee_amount, 3_000);
    }

    #[test]
    fn test_slippage_applied_to_min_amount_out() {
        let (env, client, plugin) = setup();
        let token_in = Address::generate(&env);
        let token_out = Address::generate(&env);
        let route = String::from_str(&env, "mock/pool");

        // slippage_bps = 50 (0.50%), amount_out = 2_000_000
        // min_amount_out = 2_000_000 * 9950 / 10_000 = 1_990_000
        let resp = client.get_quote(
            &plugin, &route, &token_in, &token_out,
            &1_000_000, &0, &50, &6,
        );
        assert_eq!(resp.min_amount_out, 1_990_000);
    }

    #[test]
    fn test_invalid_amount_fails() {
        let (env, client, plugin) = setup();
        let token_in = Address::generate(&env);
        let token_out = Address::generate(&env);
        let route = String::from_str(&env, "mock/pool");
        let result = client.try_get_quote(
            &plugin, &route, &token_in, &token_out,
            &0, &0, &0, &6,
        );
        assert_eq!(result, Err(Ok(QuoteError::InvalidAmount)));
    }

    #[test]
    fn test_invalid_precision_fails() {
        let (env, client, plugin) = setup();
        let token_in = Address::generate(&env);
        let token_out = Address::generate(&env);
        let route = String::from_str(&env, "mock/pool");
        let result = client.try_get_quote(
            &plugin, &route, &token_in, &token_out,
            &1_000_000, &0, &0, &0,
        );
        assert_eq!(result, Err(Ok(QuoteError::InvalidPrecision)));
    }

    #[test]
    fn test_invalid_slippage_fails() {
        let (env, client, plugin) = setup();
        let token_in = Address::generate(&env);
        let token_out = Address::generate(&env);
        let route = String::from_str(&env, "mock/pool");
        let result = client.try_get_quote(
            &plugin, &route, &token_in, &token_out,
            &1_000_000, &0, &10_001, &6,
        );
        assert_eq!(result, Err(Ok(QuoteError::InvalidSlippage)));
    }

    #[test]
    fn test_estimate_fee_normal_load() {
        let (_, client, _) = setup();
        let resp = client.estimate_fee(&1_000_000, &30, &5_000);
        assert!(!resp.surge_pricing);
        assert_eq!(resp.protocol_fee, 3_000);
        assert_eq!(resp.network_fee, 100);
        assert_eq!(resp.total_fee, 3_100);
        assert_eq!(resp.effective_fee_bps, 30);
    }

    #[test]
    fn test_estimate_fee_surge_pricing() {
        let (_, client, _) = setup();
        let resp = client.estimate_fee(&1_000_000, &30, &9_000);
        assert!(resp.surge_pricing);
        assert_eq!(resp.network_fee, 200);
        assert_eq!(resp.effective_fee_bps, 60);
    }
}
