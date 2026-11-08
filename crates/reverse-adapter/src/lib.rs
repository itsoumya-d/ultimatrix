use craft_core::{StudyMode, TargetDNA, TargetSource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeconReport {
    pub dna: TargetDNA,
    pub state_machine_summary: String,
    pub formulas: Vec<String>,
    pub tool_used: String,
}

pub trait Deconstructor: Send + Sync {
    fn name(&self) -> &'static str;
    fn deconstruct(&self, target: &TargetSource, mode: &StudyMode) -> anyhow::Result<DeconReport>;
}

pub struct ReverseSkillAdapter;
pub struct ReplicaSkillAdapter;

fn stub_dna(name: &str) -> TargetDNA {
    craft_core::TargetDNA {
        name: name.to_string(),
        source: TargetSource::Url(name.to_string()),
        core_loop: craft_core::CoreLoop::AsyncCrud,
        input_topology: craft_core::InputTopology::Pointer,
        domain_tags: vec![name.to_string()],
        schema_fields: 10,
        concurrency_class: craft_core::ConcurrencyClass::SaaSAsync,
        monetization_loop: "saas".into(),
        telemetry: craft_core::InteractionTelemetry {
            session_length_secs: 120.0,
            inputs_per_min: 30.0,
            concurrent_controls: 2,
        },
        visual_tokens: craft_core::VisualTokens {
            heading_font: "Inter".into(),
            body_font: "Inter".into(),
            mono_font: "JetBrainsMono".into(),
            accent_hex: "#000000".into(),
            radius_px: 8,
        },
        state_machine_nodes: 10,
    }
}

impl Deconstructor for ReverseSkillAdapter {
    fn name(&self) -> &'static str {
        "reverse-skill"
    }
    fn deconstruct(&self, target: &TargetSource, _mode: &StudyMode) -> anyhow::Result<DeconReport> {
        // Wire to zhaoxuya520/reverse-skill / Jakeschincariol/replica-skill
        // via CLI/MCP in production; stub returns normalized DNA.
        let label = match target {
            TargetSource::Url(u) => u.clone(),
            TargetSource::Repo(r) => r.clone(),
            TargetSource::BinaryPath(p) => p.clone(),
            TargetSource::ApkPath(p) => p.clone(),
        };
        Ok(DeconReport {
            dna: stub_dna(&label),
            state_machine_summary: "stub: wired to reverse-skill in prod".into(),
            formulas: vec![],
            tool_used: self.name().into(),
        })
    }
}

impl Deconstructor for ReplicaSkillAdapter {
    fn name(&self) -> &'static str {
        "replica-skill"
    }
    fn deconstruct(&self, target: &TargetSource, mode: &StudyMode) -> anyhow::Result<DeconReport> {
        ReverseSkillAdapter.deconstruct(target, mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stub_returns_dna() {
        let r = ReverseSkillAdapter
            .deconstruct(
                &TargetSource::Url("https://example.com".into()),
                &StudyMode::DeepStudy,
            )
            .unwrap();
        assert!(!r.dna.name.is_empty());
    }
}
