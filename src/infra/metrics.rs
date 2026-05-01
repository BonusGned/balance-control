use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::{header::CONTENT_TYPE, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use prometheus::{Encoder, GaugeVec, IntGaugeVec, Opts, Registry, TextEncoder};
use rust_decimal::prelude::ToPrimitive;
use tracing::{error, warn};

use crate::domain::model::TokenBalance;
use crate::domain::ports::MetricsRecorder;

const METRIC_LABELS: &[&str] = &["network", "account", "alias", "token"];
const METRICS_CONTENT_TYPE: &str = "text/plain; version=0.0.4; charset=utf-8";

type LabelKey = [String; 4];

#[derive(Default)]
struct CycleState {
    /// Label tuples registered in the previous completed cycle.
    previous: HashSet<LabelKey>,
    /// Label tuples observed during the current in-flight cycle.
    current: HashSet<LabelKey>,
}

pub struct PrometheusRecorder {
    balance_gauge: GaugeVec,
    below_threshold_gauge: IntGaugeVec,
    registry: Registry,
    cycle: Mutex<CycleState>,
}

impl PrometheusRecorder {
    pub fn new() -> anyhow::Result<Self> {
        let registry = Registry::new();

        let balance_gauge = GaugeVec::new(
            Opts::new("balance_control_balance", "Current token balance"),
            METRIC_LABELS,
        )?;

        let below_threshold_gauge = IntGaugeVec::new(
            Opts::new(
                "balance_control_below_threshold",
                "1 if balance is below threshold, 0 otherwise",
            ),
            METRIC_LABELS,
        )?;

        registry.register(Box::new(balance_gauge.clone()))?;
        registry.register(Box::new(below_threshold_gauge.clone()))?;

        Ok(Self {
            balance_gauge,
            below_threshold_gauge,
            registry,
            cycle: Mutex::new(CycleState::default()),
        })
    }

    fn encode(&self) -> anyhow::Result<String> {
        let encoder = TextEncoder::new();
        let families = self.registry.gather();
        let mut buf = Vec::with_capacity(4096);
        encoder.encode(&families, &mut buf)?;
        Ok(String::from_utf8(buf)?)
    }

    fn remove_series(&self, key: &LabelKey) {
        let labels: [&str; 4] = [&key[0], &key[1], &key[2], &key[3]];
        let _ = self.balance_gauge.remove_label_values(&labels);
        let _ = self.below_threshold_gauge.remove_label_values(&labels);
    }
}

impl MetricsRecorder for PrometheusRecorder {
    fn record_balance(&self, balance: &TokenBalance) {
        let labels = [
            balance.network.as_str(),
            balance.account_address.as_str(),
            balance.account_alias.as_str(),
            balance.token.as_str(),
        ];

        match balance.balance.to_f64() {
            Some(f) => self.balance_gauge.with_label_values(&labels).set(f),
            None => warn!(
                network = %balance.network,
                account = %balance.account_alias,
                token = %balance.token,
                balance = %balance.balance,
                "Failed to convert balance to f64; balance metric not updated",
            ),
        }

        self.below_threshold_gauge
            .with_label_values(&labels)
            .set(i64::from(balance.is_below_threshold()));

        let key: LabelKey = [
            balance.network.clone(),
            balance.account_address.clone(),
            balance.account_alias.clone(),
            balance.token.clone(),
        ];
        if let Ok(mut state) = self.cycle.lock() {
            state.current.insert(key);
        }
    }

    fn begin_cycle(&self) {
        if let Ok(mut state) = self.cycle.lock() {
            state.current.clear();
        }
    }

    fn end_cycle(&self) {
        let Ok(state) = self.cycle.lock() else {
            return;
        };
        let stale: Vec<LabelKey> = state.previous.difference(&state.current).cloned().collect();
        drop(state);

        for key in &stale {
            self.remove_series(key);
        }

        if let Ok(mut state) = self.cycle.lock() {
            state.previous = std::mem::take(&mut state.current);
        }
    }
}

pub async fn serve_metrics(recorder: Arc<PrometheusRecorder>, port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/metrics", get(handler))
        .with_state(recorder);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!(port, "Prometheus metrics server started");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handler(State(recorder): State<Arc<PrometheusRecorder>>) -> Response {
    match recorder.encode() {
        Ok(body) => ([(CONTENT_TYPE, METRICS_CONTENT_TYPE)], body).into_response(),
        Err(e) => {
            error!(error = %e, "Failed to encode Prometheus metrics");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn balance(network: &str, alias: &str, token: &str, amount: rust_decimal::Decimal) -> TokenBalance {
        TokenBalance {
            account_address: format!("addr-{alias}"),
            account_alias: alias.to_string(),
            network: network.to_string(),
            token: token.to_string(),
            balance: amount,
            threshold: dec!(100),
        }
    }

    #[test]
    fn prunes_stale_series_between_cycles() {
        let recorder = PrometheusRecorder::new().unwrap();

        recorder.begin_cycle();
        recorder.record_balance(&balance("evm", "A", "USDT", dec!(50)));
        recorder.record_balance(&balance("evm", "B", "USDC", dec!(200)));
        recorder.end_cycle();

        let output = recorder.encode().unwrap();
        assert!(output.contains("alias=\"A\""));
        assert!(output.contains("alias=\"B\""));

        recorder.begin_cycle();
        recorder.record_balance(&balance("evm", "A", "USDT", dec!(75)));
        recorder.end_cycle();

        let output = recorder.encode().unwrap();
        assert!(output.contains("alias=\"A\""));
        assert!(
            !output.contains("alias=\"B\""),
            "stale series for alias=B should have been removed:\n{output}"
        );
    }

    #[test]
    fn encodes_valid_prometheus_text_format() {
        let recorder = PrometheusRecorder::new().unwrap();
        recorder.begin_cycle();
        recorder.record_balance(&balance("evm", "A", "USDT", dec!(42)));
        recorder.end_cycle();

        let output = recorder.encode().unwrap();
        assert!(output.contains("# TYPE balance_control_balance gauge"));
        assert!(output.contains("# TYPE balance_control_below_threshold gauge"));
        assert!(output.contains("balance_control_balance{"));
    }
}
