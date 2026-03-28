use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::routing::get;
use prometheus::{Encoder, GaugeVec, Opts, Registry, TextEncoder};
use rust_decimal::prelude::ToPrimitive;

use crate::domain::model::TokenBalance;
use crate::domain::ports::MetricsRecorder;

pub struct PrometheusRecorder {
    balance_gauge: GaugeVec,
    below_threshold_gauge: GaugeVec,
    registry: Registry,
}

impl PrometheusRecorder {
    pub fn new() -> anyhow::Result<Self> {
        let registry = Registry::new();

        let balance_gauge = GaugeVec::new(
            Opts::new("balance_control_balance", "Current token balance"),
            &["network", "account", "alias", "token"],
        )?;

        let below_threshold_gauge = GaugeVec::new(
            Opts::new(
                "balance_control_below_threshold",
                "1 if balance is below threshold, 0 otherwise",
            ),
            &["network", "account", "alias", "token"],
        )?;

        registry.register(Box::new(balance_gauge.clone()))?;
        registry.register(Box::new(below_threshold_gauge.clone()))?;

        Ok(Self {
            balance_gauge,
            below_threshold_gauge,
            registry,
        })
    }

    fn encode_metrics(&self) -> String {
        let encoder = TextEncoder::new();
        let families = self.registry.gather();
        let mut buf = Vec::new();
        encoder.encode(&families, &mut buf).unwrap_or_default();
        String::from_utf8(buf).unwrap_or_default()
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

        if let Some(f) = balance.balance.to_f64() {
            self.balance_gauge.with_label_values(&labels).set(f);
        }

        self.below_threshold_gauge.with_label_values(&labels).set(
            if balance.is_below_threshold() {
                1.0
            } else {
                0.0
            },
        );
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

async fn handler(State(recorder): State<Arc<PrometheusRecorder>>) -> String {
    recorder.encode_metrics()
}
