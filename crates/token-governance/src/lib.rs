use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub max_per_stage: u32,
    pub used: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Telemetry {
    pub per_stage: HashMap<String, u32>,
    pub total_cost_usd: f64,
}

#[derive(Debug, Clone)]
pub struct Governance {
    pub budgets: HashMap<String, u32>,
    pub telemetry: Telemetry,
}

impl Default for Governance {
    fn default() -> Self {
        let mut budgets = HashMap::new();
        for s in [
            "ingestion",
            "deconstruction",
            "conflict",
            "mining",
            "simulation",
            "synthesis",
            "approval",
            "build",
        ] {
            budgets.insert(s.to_string(), 50_000);
        }
        Self {
            budgets,
            telemetry: Telemetry::default(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GovError {
    #[error("budget exceeded for stage {0}: used {1} > max {2}")]
    BudgetExceeded(String, u32, u32),
    #[error("structural validation failed: {0}")]
    Structural(String),
    #[error("cache miss")]
    CacheMiss,
}

pub fn estimate_tokens(text: &str) -> u32 {
    // LLMLingua-style approximation: ~4 chars/token.
    ((text.len() as f32) / 4.0).ceil() as u32
}

impl Governance {
    pub fn check(&self, stage: &str, tokens: u32) -> Result<(), GovError> {
        let used = self.telemetry.per_stage.get(stage).copied().unwrap_or(0);
        let max = self.budgets.get(stage).copied().unwrap_or(50_000);
        if used + tokens > max {
            Err(GovError::BudgetExceeded(stage.into(), used + tokens, max))
        } else {
            Ok(())
        }
    }

    pub fn record(&mut self, stage: &str, tokens: u32) {
        *self.telemetry.per_stage.entry(stage.into()).or_insert(0) += tokens;
        self.telemetry.total_cost_usd += tokens as f64 * 0.000003;
    }

    /// LLMLingua compression stub: keeps head + tail, reports ratio.
    /// Production delegates to microsoft/LLMLingua for up to 20x compression.
    pub fn compress(&self, text: &str, target_ratio: f32) -> (String, f32) {
        let original = estimate_tokens(text);
        if text.len() <= 400 {
            return (text.to_string(), 1.0);
        }
        let keep = ((text.len() as f32) * target_ratio) as usize;
        let half = keep / 2;
        let out = format!("{}…{}", &text[..half], &text[text.len() - half..]);
        let ratio = if original == 0 {
            1.0
        } else {
            estimate_tokens(&out) as f32 / original as f32
        };
        (out, ratio)
    }

    /// Guidance-style structural lock: reject malformed JSON.
    pub fn validate_structural(&self, json: &str) -> Result<serde_json::Value, GovError> {
        serde_json::from_str(json).map_err(|e| GovError::Structural(e.to_string()))
    }

    /// Guidance schema lock: require object with "name" and "score" 0-100.
    pub fn guidance_validate(&self, json: &str) -> Result<serde_json::Value, GovError> {
        let v: serde_json::Value =
            serde_json::from_str(json).map_err(|e| GovError::Structural(e.to_string()))?;
        let obj = v
            .as_object()
            .ok_or_else(|| GovError::Structural("not an object".into()))?;
        if obj.get("name").and_then(|x| x.as_str()).is_none() {
            return Err(GovError::Structural("missing name".into()));
        }
        let score = obj.get("score").and_then(|x| x.as_u64());
        match score {
            Some(s) if s <= 100 => {}
            _ => return Err(GovError::Structural("score 0-100 required".into())),
        }
        Ok(v)
    }

    /// Dynamic model routing hint: cheap stage -> small model, synthesis -> large.
    pub fn route_model(&self, stage: &str) -> &'static str {
        match stage {
            "synthesis" | "approval" => "large-reasoning",
            "mining" | "simulation" => "mid-balanced",
            _ => "small-fast",
        }
    }

    pub fn semantic_cache_lookup(cache: &HashMap<String, String>, key: &str) -> Option<String> {
        cache.get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn estimate() {
        assert!(estimate_tokens("hello world") >= 1);
    }
    #[test]
    fn budget_blocks() {
        let g = Governance::default();
        assert!(g.check("ingestion", 999_999).is_err());
        assert!(g.check("ingestion", 10).is_ok());
    }
    #[test]
    fn compress_shrinks() {
        let g = Governance::default();
        let big = "abc ".repeat(500);
        let (out, r) = g.compress(&big, 0.25);
        assert!(out.len() < big.len());
        assert!(r < 1.0);
    }
}
