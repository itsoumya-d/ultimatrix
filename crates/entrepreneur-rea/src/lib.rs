use craft_core::TargetDNA;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntrepreneurLedger {
    pub top_bugs: Vec<String>,
    pub unmet_demands: Vec<String>,
    pub thesis: String,
}

pub trait PainMiner: Send + Sync {
    fn miner_name(&self) -> &'static str;
    fn mine(
        &self,
        target: &TargetDNA,
        competitors: &[TargetDNA],
    ) -> anyhow::Result<EntrepreneurLedger>;
}

pub struct ReaAdapter;
pub struct ReviewScraperAdapter;

impl PainMiner for ReaAdapter {
    fn miner_name(&self) -> &'static str {
        "morluto/rea"
    }
    fn mine(
        &self,
        target: &TargetDNA,
        _competitors: &[TargetDNA],
    ) -> anyhow::Result<EntrepreneurLedger> {
        // Production: scrapes GitHub Issues, App Store/Play Store 1-3 star,
        // Reddit, HN via appstore-review-cli + requesthunt adapters.
        Ok(EntrepreneurLedger {
            top_bugs: vec![format!("stub bug for {}", target.name)],
            unmet_demands: vec!["stub demand".into()],
            thesis: format!(
                "Fixing top complaints for {} creates category-leadership opening.",
                target.name
            ),
        })
    }
}

impl PainMiner for ReviewScraperAdapter {
    fn miner_name(&self) -> &'static str {
        "review-scrapers"
    }
    fn mine(
        &self,
        target: &TargetDNA,
        competitors: &[TargetDNA],
    ) -> anyhow::Result<EntrepreneurLedger> {
        ReaAdapter.mine(target, competitors).map(|mut l| {
            l.thesis.push_str(" [+scrapers]");
            l
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ledger_nonempty() {
        let dna = craft_core::TargetDNA {
            name: "X".into(),
            source: craft_core::TargetSource::Url("x".into()),
            core_loop: craft_core::CoreLoop::AsyncCrud,
            input_topology: craft_core::InputTopology::Pointer,
            domain_tags: vec!["x".into()],
            schema_fields: 5,
            concurrency_class: craft_core::ConcurrencyClass::SoloLocal,
            monetization_loop: "saas".into(),
            telemetry: craft_core::InteractionTelemetry {
                session_length_secs: 10.0,
                inputs_per_min: 10.0,
                concurrent_controls: 1,
            },
            visual_tokens: craft_core::VisualTokens {
                heading_font: "Inter".into(),
                body_font: "Inter".into(),
                mono_font: "Mono".into(),
                accent_hex: "#000".into(),
                radius_px: 4,
            },
            state_machine_nodes: 5,
        };
        let l = ReaAdapter.mine(&dna, &[]).unwrap();
        assert!(!l.top_bugs.is_empty());
    }
}
