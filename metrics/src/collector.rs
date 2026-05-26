<<<<<<< feat/400-401-402-403
/// Metrics collectors for router-execution and router-quote contracts.
///
/// This module defines the Prometheus metric descriptors and the scrape logic
/// for the two off-chain contracts. It is designed to be integrated into the
/// main metrics exporter binary alongside the existing contract collectors.
///
/// ## Metrics exposed
///
/// ### router-execution
/// | Metric | Type | Description |
/// |---|---|---|
/// | `router_execution_total_executions` | Counter | Cumulative successful executions |
/// | `router_execution_total_errors` | Counter | Cumulative execution errors |
/// | `router_execution_error_rate` | Gauge | errors / (executions + errors) |
/// | `router_execution_max_retries` | Gauge | Configured max retry cap |
///
/// ### router-quote
/// | Metric | Type | Description |
/// |---|---|---|
/// | `router_quote_total_quotes` | Counter | Cumulative `get_quote` calls |
/// | `router_quote_total_fee_estimates` | Counter | Cumulative `estimate_fee` calls |
/// | `router_quote_surge_pricing_active` | Gauge | 1 if last estimate had surge pricing |
///
/// ## Integration
///
/// Add the following to your exporter's main scrape loop:
///
/// ```rust,ignore
/// use crate::collector::{ExecutionCollector, QuoteCollector};
///
/// let exec = ExecutionCollector::new(&rpc_client, &execution_contract_id);
/// let quote = QuoteCollector::new(&rpc_client, &quote_contract_id);
///
/// // In your scrape handler:
/// exec.collect(&mut registry).await?;
/// quote.collect(&mut registry).await?;
/// ```
use std::collections::HashMap;

/// Scraped metrics from router-execution.
#[derive(Debug, Default)]
pub struct ExecutionMetrics {
    /// Cumulative successful executions (`TotalExecutions` storage key).
    pub total_executions: u64,
    /// Cumulative errors (`TotalErrors` storage key).
    pub total_errors: u64,
    /// Configured max retry cap (`MaxRetries` storage key).
    pub max_retries: u32,
}

impl ExecutionMetrics {
    /// Error rate as a fraction (0.0–1.0). Returns 0.0 if no calls have been made.
    pub fn error_rate(&self) -> f64 {
        let total = self.total_executions + self.total_errors;
        if total == 0 {
            0.0
        } else {
            self.total_errors as f64 / total as f64
        }
    }

    /// Render metrics in Prometheus text exposition format.
    pub fn to_prometheus(&self) -> String {
        let mut out = String::new();

        out.push_str("# HELP router_execution_total_executions Cumulative successful executions\n");
        out.push_str("# TYPE router_execution_total_executions counter\n");
        out.push_str(&format!(
            "router_execution_total_executions {}\n",
            self.total_executions
        ));

        out.push_str("# HELP router_execution_total_errors Cumulative execution errors\n");
        out.push_str("# TYPE router_execution_total_errors counter\n");
        out.push_str(&format!(
            "router_execution_total_errors {}\n",
            self.total_errors
        ));

        out.push_str("# HELP router_execution_error_rate Fraction of calls that resulted in an error\n");
        out.push_str("# TYPE router_execution_error_rate gauge\n");
        out.push_str(&format!(
            "router_execution_error_rate {:.6}\n",
            self.error_rate()
        ));

        out.push_str("# HELP router_execution_max_retries Configured maximum retry cap\n");
        out.push_str("# TYPE router_execution_max_retries gauge\n");
        out.push_str(&format!(
            "router_execution_max_retries {}\n",
            self.max_retries
        ));

        out
    }
}

/// Scraped metrics from router-quote.
#[derive(Debug, Default)]
pub struct QuoteMetrics {
    /// Cumulative `get_quote` invocations.
    pub total_quotes: u64,
    /// Cumulative `estimate_fee` invocations.
    pub total_fee_estimates: u64,
    /// Whether the most recent fee estimate applied surge pricing.
    pub surge_pricing_active: bool,
}

impl QuoteMetrics {
    /// Render metrics in Prometheus text exposition format.
    pub fn to_prometheus(&self) -> String {
        let mut out = String::new();

        out.push_str("# HELP router_quote_total_quotes Cumulative get_quote invocations\n");
        out.push_str("# TYPE router_quote_total_quotes counter\n");
        out.push_str(&format!(
            "router_quote_total_quotes {}\n",
            self.total_quotes
        ));

        out.push_str("# HELP router_quote_total_fee_estimates Cumulative estimate_fee invocations\n");
        out.push_str("# TYPE router_quote_total_fee_estimates counter\n");
        out.push_str(&format!(
            "router_quote_total_fee_estimates {}\n",
            self.total_fee_estimates
        ));

        out.push_str("# HELP router_quote_surge_pricing_active 1 if the last fee estimate applied surge pricing\n");
        out.push_str("# TYPE router_quote_surge_pricing_active gauge\n");
        out.push_str(&format!(
            "router_quote_surge_pricing_active {}\n",
            if self.surge_pricing_active { 1 } else { 0 }
        ));

        out
    }
}

/// Scrapes router-execution metrics from the Soroban RPC.
///
/// Reads `TotalExecutions`, `TotalErrors`, and `MaxRetries` from the
/// contract's instance storage via `getLedgerEntries`.
pub struct ExecutionCollector {
    rpc_url: String,
    contract_id: String,
}

impl ExecutionCollector {
    pub fn new(rpc_url: impl Into<String>, contract_id: impl Into<String>) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            contract_id: contract_id.into(),
        }
    }

    /// Scrape the contract and return the current metrics.
    ///
    /// In a production implementation this calls `getLedgerEntries` with the
    /// XDR-encoded storage keys for `TotalExecutions`, `TotalErrors`, and
    /// `MaxRetries`. The placeholder below returns zeroed metrics and should
    /// be replaced with real RPC calls using the `stellar-xdr` crate.
    pub async fn scrape(&self) -> Result<ExecutionMetrics, String> {
        // TODO: replace with real getLedgerEntries call
        // Keys to fetch:
        //   DataKey::TotalExecutions  → u64
        //   DataKey::TotalErrors      → u64
        //   DataKey::MaxRetries       → u32
        Ok(ExecutionMetrics::default())
    }
}

/// Scrapes router-quote metrics from the Soroban RPC.
pub struct QuoteCollector {
    rpc_url: String,
    contract_id: String,
}

impl QuoteCollector {
    pub fn new(rpc_url: impl Into<String>, contract_id: impl Into<String>) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            contract_id: contract_id.into(),
        }
    }

    /// Scrape the contract and return the current metrics.
    ///
    /// router-quote does not currently persist counters in storage — it emits
    /// `quote_generated` and `fee_estimated` events instead. A production
    /// implementation should subscribe to these events via `getEvents` and
    /// maintain counters off-chain, or add storage counters to the contract.
    pub async fn scrape(&self) -> Result<QuoteMetrics, String> {
        // TODO: subscribe to quote_generated and fee_estimated events via getEvents
        // and maintain running counters.
        Ok(QuoteMetrics::default())
    }
=======
//! Background scrape loop.
//!
//! The [`Collector`] spawns a `tokio` task that wakes up every
//! `scrape_interval_secs` seconds, queries each configured router contract
//! via the Soroban RPC, and updates the Prometheus gauges / counters.
//!
//! ## Scraping strategy
//!
//! Soroban contracts store state in on-chain ledger entries.  The cleanest
//! way to read that state from off-chain is to call the contract's view
//! functions via `simulateTransaction`.  This exporter calls:
//!
//! - `router-core`:       `total_routed()`, `is_paused()`, `get_all_routes()`
//!                        + `get_route(name)` for each route
//! - `router-middleware`: `total_calls()`, `get_configured_routes()`
//!                        + `circuit_breaker_state(route)` for each route
//! - `router-registry`:   `get_all_names()` (total count)
//!
//! Each contract scrape is timed and any error increments the
//! `router_scrape_errors_total` counter for that contract label.

use std::time::Instant;

use anyhow::Result;
use tracing::{error, info, warn};

use crate::cli::Args;
use crate::metrics::RouterMetrics;
use crate::rpc::{RpcClient, SorobanRpcClient};

/// Drives the periodic scrape loop.
#[derive(Clone)]
pub struct Collector {
    args: Args,
    metrics: RouterMetrics,
}

impl Collector {
    pub fn new(args: Args, metrics: RouterMetrics) -> Self {
        Self { args, metrics }
    }

    /// Run forever, scraping on the configured interval.
    pub async fn run(self) {
        let interval = tokio::time::Duration::from_secs(self.args.scrape_interval_secs);
        info!(
            interval_secs = self.args.scrape_interval_secs,
            "scrape loop started"
        );

        let client = match SorobanRpcClient::new(&self.args.rpc_url, self.args.rpc_timeout_secs) {
            Ok(c) => c,
            Err(e) => {
                error!("failed to create RPC client: {e:#}");
                return;
            }
        };

        loop {
            let cycle_ok = self.scrape_all(&client).await;
            self.metrics.up.set(if cycle_ok { 1.0 } else { 0.0 });
            tokio::time::sleep(interval).await;
        }
    }

    /// Scrape all configured contracts.  Returns `true` if every scrape
    /// succeeded, `false` if any failed.
    async fn scrape_all(&self, client: &dyn RpcClient) -> bool {
        let mut all_ok = true;

        if !self.args.core_contract_id.is_empty() {
            if let Err(e) = self.scrape_core(client, &self.args.core_contract_id).await {
                warn!(contract = %self.args.core_contract_id, "core scrape failed: {e:#}");
                self.metrics
                    .scrape_errors_total
                    .with_label_values(&[&self.args.core_contract_id])
                    .inc();
                all_ok = false;
            }
        }

        if !self.args.middleware_contract_id.is_empty() {
            if let Err(e) = self
                .scrape_middleware(client, &self.args.middleware_contract_id)
                .await
            {
                warn!(contract = %self.args.middleware_contract_id, "middleware scrape failed: {e:#}");
                self.metrics
                    .scrape_errors_total
                    .with_label_values(&[&self.args.middleware_contract_id])
                    .inc();
                all_ok = false;
            }
        }

        if !self.args.registry_contract_id.is_empty() {
            if let Err(e) = self
                .scrape_registry(client, &self.args.registry_contract_id)
                .await
            {
                warn!(contract = %self.args.registry_contract_id, "registry scrape failed: {e:#}");
                self.metrics
                    .scrape_errors_total
                    .with_label_values(&[&self.args.registry_contract_id])
                    .inc();
                all_ok = false;
            }
        }

        all_ok
    }

    // ── router-core ───────────────────────────────────────────────────────────

    async fn scrape_core(&self, client: &dyn RpcClient, contract_id: &str) -> Result<()> {
        let start = Instant::now();
        info!(contract_id, "scraping router-core");

        // 1. total_routed
        let total_routed = client.call_u64(contract_id, "total_routed").await?;
        self.metrics
            .core_total_routed
            .with_label_values(&[contract_id])
            .set(total_routed as f64);

        // 2. is_paused (router-core exposes this via storage; we call set_paused
        //    indirectly — the contract stores a `Paused` bool in instance storage.
        //    We read it via a helper view function if available, otherwise we
        //    attempt to resolve a non-existent route and check for RouterPaused.)
        //
        //    router-core does not expose a dedicated `is_paused()` view function
        //    in the current implementation, so we use `get_route` on a sentinel
        //    name and interpret the error.  A cleaner approach is to add a
        //    `is_paused()` view function to the contract (tracked separately).
        //
        //    For now we record 0 (unknown / not paused) and note the limitation.
        self.metrics
            .core_paused
            .with_label_values(&[contract_id])
            .set(0.0); // updated below if the RPC call succeeds

        // 3. get_all_routes → per-route paused state
        let routes = client
            .call_string_vec(contract_id, "get_all_routes")
            .await?;
        for route in &routes {
            // get_route returns a RouteEntry; we check the `paused` field.
            // The JSON representation of a Soroban struct is a map of field names.
            let route_result = client
                .simulate_invoke(contract_id, "get_route", vec![encode_string_arg(route)])
                .await;

            match route_result {
                Ok(val) => {
                    let paused = extract_route_paused(&val).unwrap_or(false);
                    self.metrics
                        .core_route_paused
                        .with_label_values(&[contract_id, route])
                        .set(if paused { 1.0 } else { 0.0 });
                }
                Err(e) => {
                    warn!(contract_id, route, "failed to get route state: {e:#}");
                }
            }
        }

        let elapsed = start.elapsed().as_secs_f64();
        self.metrics
            .scrape_duration_seconds
            .with_label_values(&[contract_id])
            .observe(elapsed);

        info!(
            contract_id,
            elapsed_secs = elapsed,
            routes = routes.len(),
            total_routed,
            "core scrape done"
        );
        Ok(())
    }

    // ── router-middleware ─────────────────────────────────────────────────────

    async fn scrape_middleware(&self, client: &dyn RpcClient, contract_id: &str) -> Result<()> {
        let start = Instant::now();
        info!(contract_id, "scraping router-middleware");

        // 1. total_calls
        let total_calls = client.call_u64(contract_id, "total_calls").await?;
        self.metrics
            .middleware_total_calls
            .with_label_values(&[contract_id])
            .set(total_calls as f64);

        // 2. Per-route circuit breaker state
        let routes = client
            .call_string_vec(contract_id, "get_configured_routes")
            .await?;

        for route in &routes {
            let cb_result = client
                .simulate_invoke(
                    contract_id,
                    "circuit_breaker_state",
                    vec![encode_string_arg(route)],
                )
                .await;

            match cb_result {
                Ok(val) => {
                    let (is_open, failure_count) =
                        extract_circuit_breaker_state(&val).unwrap_or((false, 0));
                    self.metrics
                        .middleware_circuit_open
                        .with_label_values(&[contract_id, route])
                        .set(if is_open { 1.0 } else { 0.0 });
                    self.metrics
                        .middleware_failure_count
                        .with_label_values(&[contract_id, route])
                        .set(failure_count as f64);
                }
                Err(e) => {
                    warn!(
                        contract_id,
                        route, "failed to get circuit breaker state: {e:#}"
                    );
                }
            }
        }

        let elapsed = start.elapsed().as_secs_f64();
        self.metrics
            .scrape_duration_seconds
            .with_label_values(&[contract_id])
            .observe(elapsed);

        info!(
            contract_id,
            elapsed_secs = elapsed,
            routes = routes.len(),
            total_calls,
            "middleware scrape done"
        );
        Ok(())
    }

    // ── router-registry ───────────────────────────────────────────────────────

    async fn scrape_registry(&self, client: &dyn RpcClient, contract_id: &str) -> Result<()> {
        let start = Instant::now();
        info!(contract_id, "scraping router-registry");

        // get_all_names returns Vec<String> of registered contract names
        let names = client.call_string_vec(contract_id, "get_all_names").await?;

        self.metrics
            .registry_total_names
            .with_label_values(&[contract_id])
            .set(names.len() as f64);

        let elapsed = start.elapsed().as_secs_f64();
        self.metrics
            .scrape_duration_seconds
            .with_label_values(&[contract_id])
            .observe(elapsed);

        info!(
            contract_id,
            elapsed_secs = elapsed,
            total_names = names.len(),
            "registry scrape done"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::MockRpcClient;
    use prometheus::Registry;
    use serde_json::json;

    fn make_collector(
        core: &str,
        middleware: &str,
        registry_id: &str,
    ) -> (Collector, RouterMetrics) {
        let reg = Registry::new();
        let metrics = RouterMetrics::new(&reg).unwrap();
        let args = Args {
            rpc_url: String::new(),
            network_passphrase: String::new(),
            core_contract_id: core.to_string(),
            middleware_contract_id: middleware.to_string(),
            registry_contract_id: registry_id.to_string(),
            scrape_interval_secs: 15,
            listen: "0.0.0.0:9090".to_string(),
            rpc_timeout_secs: 10,
        };
        let collector = Collector::new(args, metrics.clone());
        (collector, metrics)
    }

    #[tokio::test]
    async fn test_scrape_core_updates_metrics() {
        let (collector, metrics) = make_collector("CORE_ID", "", "");

        let mock = MockRpcClient::new()
            .with_u64("CORE_ID", "total_routed", 42)
            .with_string_vec("CORE_ID", "get_all_routes", vec![]);

        let ok = collector.scrape_all(&mock).await;
        assert!(ok);

        let val = metrics
            .core_total_routed
            .with_label_values(&["CORE_ID"])
            .get();
        assert_eq!(val, 42.0);
    }

    #[tokio::test]
    async fn test_scrape_middleware_updates_metrics() {
        let (collector, metrics) = make_collector("", "MW_ID", "");

        let mock = MockRpcClient::new()
            .with_u64("MW_ID", "total_calls", 7)
            .with_string_vec("MW_ID", "get_configured_routes", vec![]);

        let ok = collector.scrape_all(&mock).await;
        assert!(ok);

        let val = metrics
            .middleware_total_calls
            .with_label_values(&["MW_ID"])
            .get();
        assert_eq!(val, 7.0);
    }

    #[tokio::test]
    async fn test_scrape_registry_updates_metrics() {
        let (collector, metrics) = make_collector("", "", "REG_ID");

        let mock = MockRpcClient::new().with_string_vec(
            "REG_ID",
            "get_all_names",
            vec!["oracle".to_string(), "vault".to_string()],
        );

        let ok = collector.scrape_all(&mock).await;
        assert!(ok);

        let val = metrics
            .registry_total_names
            .with_label_values(&["REG_ID"])
            .get();
        assert_eq!(val, 2.0);
    }

    #[tokio::test]
    async fn test_scrape_failure_returns_false_and_increments_error_counter() {
        let (collector, metrics) = make_collector("CORE_ID", "", "");

        // Mock returns no response → scrape_core will fail
        let mock = MockRpcClient::new();

        let ok = collector.scrape_all(&mock).await;
        assert!(!ok);

        let errors = metrics
            .scrape_errors_total
            .with_label_values(&["CORE_ID"])
            .get();
        assert_eq!(errors, 1.0);
    }

    #[tokio::test]
    async fn test_scrape_core_with_routes_and_circuit_breaker() {
        let (collector, metrics) = make_collector("CORE_ID", "MW_ID", "");

        let mock = MockRpcClient::new()
            .with_u64("CORE_ID", "total_routed", 100)
            .with_string_vec(
                "CORE_ID",
                "get_all_routes",
                vec!["oracle".to_string()],
            )
            .with_simulate(
                "CORE_ID",
                "get_route",
                json!({ "results": [{ "retval": { "paused": false } }] }),
            )
            .with_u64("MW_ID", "total_calls", 50)
            .with_string_vec(
                "MW_ID",
                "get_configured_routes",
                vec!["oracle".to_string()],
            )
            .with_simulate(
                "MW_ID",
                "circuit_breaker_state",
                json!({
                    "results": [{
                        "retval": {
                            "some": { "is_open": true, "failure_count": 3, "opened_at": 1000 }
                        }
                    }]
                }),
            );

        let ok = collector.scrape_all(&mock).await;
        assert!(ok);

        assert_eq!(
            metrics
                .core_total_routed
                .with_label_values(&["CORE_ID"])
                .get(),
            100.0
        );
        assert_eq!(
            metrics
                .middleware_circuit_open
                .with_label_values(&["MW_ID", "oracle"])
                .get(),
            1.0
        );
        assert_eq!(
            metrics
                .middleware_failure_count
                .with_label_values(&["MW_ID", "oracle"])
                .get(),
            3.0
        );
    }
}

/// Encode a plain string as a base64 XDR `ScVal::String` argument.
///
/// This is a placeholder — a real implementation would use the `stellar-xdr`
/// crate to produce the correct XDR encoding.
fn encode_string_arg(s: &str) -> String {
    // Base64-encode the raw UTF-8 bytes as a minimal stand-in.
    // Replace with proper ScVal XDR encoding in production.
    use std::fmt::Write;
    let mut out = String::new();
    for b in s.as_bytes() {
        write!(out, "{b:02x}").ok();
    }
    out
}

/// Extract the `paused` field from a `RouteEntry` JSON value returned by
/// `simulateTransaction`.
fn extract_route_paused(val: &serde_json::Value) -> Option<bool> {
    // The Soroban RPC returns struct fields as a JSON map.
    // RouteEntry { address, name, paused, updated_by, metadata }
    val.get("results")
        .and_then(|r| r.get(0))
        .and_then(|r| r.get("retval"))
        .and_then(|v| v.get("paused"))
        .and_then(|p| p.as_bool())
        .or_else(|| val.get("paused").and_then(|p| p.as_bool()))
}

/// Extract `(is_open, failure_count)` from a `CircuitBreakerState` JSON value.
fn extract_circuit_breaker_state(val: &serde_json::Value) -> Option<(bool, u32)> {
    let retval = val
        .get("results")
        .and_then(|r| r.get(0))
        .and_then(|r| r.get("retval"))
        .unwrap_or(val);

    // Handle Option<CircuitBreakerState> — None means no state recorded yet
    if retval.is_null() || retval.get("none").is_some() {
        return Some((false, 0));
    }

    let state = retval.get("some").unwrap_or(retval);
    let is_open = state
        .get("is_open")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let failure_count = state
        .get("failure_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    Some((is_open, failure_count))
>>>>>>> main
}

#[cfg(test)]
mod tests {
    use super::*;
<<<<<<< feat/400-401-402-403

    #[test]
    fn test_execution_metrics_error_rate_zero_when_no_calls() {
        let m = ExecutionMetrics::default();
        assert_eq!(m.error_rate(), 0.0);
    }

    #[test]
    fn test_execution_metrics_error_rate_calculated() {
        let m = ExecutionMetrics {
            total_executions: 90,
            total_errors: 10,
            max_retries: 2,
        };
        assert!((m.error_rate() - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_execution_metrics_prometheus_output_contains_all_metrics() {
        let m = ExecutionMetrics {
            total_executions: 100,
            total_errors: 5,
            max_retries: 3,
        };
        let output = m.to_prometheus();
        assert!(output.contains("router_execution_total_executions 100"));
        assert!(output.contains("router_execution_total_errors 5"));
        assert!(output.contains("router_execution_error_rate"));
        assert!(output.contains("router_execution_max_retries 3"));
    }

    #[test]
    fn test_quote_metrics_prometheus_output_contains_all_metrics() {
        let m = QuoteMetrics {
            total_quotes: 42,
            total_fee_estimates: 17,
            surge_pricing_active: true,
        };
        let output = m.to_prometheus();
        assert!(output.contains("router_quote_total_quotes 42"));
        assert!(output.contains("router_quote_total_fee_estimates 17"));
        assert!(output.contains("router_quote_surge_pricing_active 1"));
    }

    #[test]
    fn test_surge_pricing_inactive_renders_as_zero() {
        let m = QuoteMetrics {
            surge_pricing_active: false,
            ..Default::default()
        };
        assert!(m.to_prometheus().contains("router_quote_surge_pricing_active 0"));
=======
    use serde_json::json;

    #[test]
    fn test_extract_route_paused_true() {
        let val = json!({
            "results": [{ "retval": { "paused": true } }]
        });
        assert_eq!(extract_route_paused(&val), Some(true));
    }

    #[test]
    fn test_extract_route_paused_false() {
        let val = json!({ "paused": false });
        assert_eq!(extract_route_paused(&val), Some(false));
    }

    #[test]
    fn test_extract_circuit_breaker_open() {
        let val = json!({
            "results": [{
                "retval": {
                    "some": {
                        "is_open": true,
                        "failure_count": 5,
                        "opened_at": 1000
                    }
                }
            }]
        });
        assert_eq!(extract_circuit_breaker_state(&val), Some((true, 5)));
    }

    #[test]
    fn test_extract_circuit_breaker_none() {
        let val = json!({
            "results": [{ "retval": null }]
        });
        assert_eq!(extract_circuit_breaker_state(&val), Some((false, 0)));
    }

    #[test]
    fn test_extract_circuit_breaker_closed() {
        let val = json!({
            "results": [{
                "retval": {
                    "some": {
                        "is_open": false,
                        "failure_count": 2,
                        "opened_at": 0
                    }
                }
            }]
        });
        assert_eq!(extract_circuit_breaker_state(&val), Some((false, 2)));
>>>>>>> main
    }
}
