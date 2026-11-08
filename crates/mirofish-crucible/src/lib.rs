use entrepreneur_rea::EntrepreneurLedger;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    pub epochs: u32,
    pub agents_per_epoch: u32,
    pub seed: u64,
    pub pressure_bugs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub task_velocity: f32,
    pub churn_rate: f32,
    pub cognitive_friction: f32,
    pub viral_retention: f32,
    pub edge_case_survival: f32,
}

#[derive(Debug, thiserror::Error)]
pub enum MiroFishError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("simulation failed: {0}")]
    Sim(String),
}

pub struct MiroFishClient {
    pub base_url: String,
    pub dry_run: bool,
    http: reqwest::Client,
}

impl Default for MiroFishClient {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8000".into(),
            dry_run: true,
            http: reqwest::Client::new(),
        }
    }
}

impl MiroFishClient {
    /// Isolated HTTP/gRPC/MCP boundary — MiroFish is AGPL-3.0 and must never
    /// be compiled into the MIT/Apache-2.0 binary. Production transport
    /// POSTs compressed TargetDNA+EntrepreneurLedger to base_url.
    pub async fn run_simulation(
        &self,
        config: &EvolutionConfig,
        _ledger: &EntrepreneurLedger,
    ) -> anyhow::Result<SimulationMetrics> {
        if self.dry_run {
            // Deterministic pseudo-metrics from seed for tests/offline.
            let s = config.seed as f32 / 100.0;
            return Ok(SimulationMetrics {
                task_velocity: 0.85 + (s % 0.1),
                churn_rate: 0.08,
                cognitive_friction: 0.45,
                viral_retention: 0.72,
                edge_case_survival: 0.91,
            });
        }
        self.run_simulation_http(config, _ledger)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn run_simulation_http(
        &self,
        config: &EvolutionConfig,
        ledger: &EntrepreneurLedger,
    ) -> anyhow::Result<SimulationMetrics> {
        let payload = serde_json::json!({"config": config, "ledger": ledger});
        let resp = self
            .http
            .post(format!("{}/simulate", self.base_url.trim_end_matches('/')))
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!(MiroFishError::Http(e)))?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(MiroFishError::Sim(format!(
                "status {}",
                resp.status()
            ))));
        }
        let metrics: SimulationMetrics = resp.json().await?;
        Ok(metrics)
    }

    pub async fn health_async(&self) -> bool {
        if self.dry_run {
            return true;
        }
        self.http
            .get(format!("{}/health", self.base_url.trim_end_matches('/')))
            .send()
            .await
            .is_ok_and(|r| r.status().is_success())
    }

    pub fn health(&self) -> bool {
        self.dry_run
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn dry_run_metrics() {
        let c = MiroFishClient::default();
        let cfg = EvolutionConfig {
            epochs: 10,
            agents_per_epoch: 100,
            seed: 7,
            pressure_bugs: vec!["crash".into()],
        };
        let m = c
            .run_simulation(&cfg, &EntrepreneurLedger::default())
            .await
            .unwrap();
        assert!(m.edge_case_survival > 0.5);
    }
}
