use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TargetSource {
    Url(String),
    Repo(String),
    BinaryPath(String),
    ApkPath(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunMode {
    SingleUltimateEvolution,
    OmniverseFusion,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StudyMode {
    VisibleOnly,
    DeepStudy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CoreLoop {
    RealTime { frame_ms: f32 },
    TurnBased { avg_turn_secs: f32 },
    AsyncCrud,
    Mixed { frame_ms: f32, avg_turn_secs: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InputTopology {
    Pointer,
    Keybinds,
    TouchGestures,
    TextHeavy,
    Mixed(Vec<String>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConcurrencyClass {
    SoloLocal = 0,
    OneVsOne = 1,
    SmallMultiplayer = 2,
    AuthoritativeMassive = 3,
    OfflineCrud = 4,
    SaaSAsync = 5,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InteractionTelemetry {
    pub session_length_secs: f32,
    pub inputs_per_min: f32,
    pub concurrent_controls: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisualTokens {
    pub heading_font: String,
    pub body_font: String,
    pub mono_font: String,
    pub accent_hex: String,
    pub radius_px: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TargetDNA {
    pub name: String,
    pub source: TargetSource,
    pub core_loop: CoreLoop,
    pub input_topology: InputTopology,
    pub domain_tags: Vec<String>,
    pub schema_fields: usize,
    pub concurrency_class: ConcurrencyClass,
    pub monetization_loop: String,
    pub telemetry: InteractionTelemetry,
    pub visual_tokens: VisualTokens,
    pub state_machine_nodes: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Stage {
    Ingestion,
    Deconstruction,
    CompatibilityGate,
    PainPointMining,
    EvolutionSimulation,
    Synthesis,
    Approval,
    Build,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StageState {
    pub stage: Stage,
    pub completed: bool,
    pub notes: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("invalid target count: {0}")]
    InvalidTargetCount(usize),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

// Re-exported trait shape for craft-anything integration.
// Upstream craft-anything has ~13 crates (engine, disasm, js, mcp, etc)
// and 314 CLI/MCP tools. This crate is the pure-Rust orchestrator
// boundary so the upstream tools can be wired via path dependencies
// without pulling the egui/oxc stack into the fusion core.
pub trait StudyTool: Send + Sync {
    fn tool_name(&self) -> &'static str;
    fn study(&self, target: &TargetSource, mode: &StudyMode) -> Result<TargetDNA, CoreError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunConfig {
    pub mode: RunMode,
    pub study_mode: StudyMode,
    pub targets: Vec<TargetSource>,
    pub seed: u64,
}

impl RunConfig {
    pub fn new(
        mode: RunMode,
        study_mode: StudyMode,
        targets: Vec<TargetSource>,
    ) -> Result<Self, CoreError> {
        match mode {
            RunMode::SingleUltimateEvolution if targets.len() != 1 => {
                Err(CoreError::InvalidTargetCount(targets.len()))
            }
            RunMode::OmniverseFusion if !(2..=10).contains(&targets.len()) => {
                Err(CoreError::InvalidTargetCount(targets.len()))
            }
            _ => Ok(Self {
                mode,
                study_mode,
                targets,
                seed: 42,
            }),
        }
    }
}

/// Strict 9-stage (8 persisted stages) deterministic state machine.
/// Craft-anything 314 tools plug in at Deconstruction + Build.
#[derive(Debug, Clone)]
pub struct Orchestrator {
    pub current: Stage,
    pub history: Vec<StageState>,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self {
            current: Stage::Ingestion,
            history: Vec::new(),
        }
    }
}

impl Orchestrator {
    pub fn stage_order() -> Vec<Stage> {
        vec![
            Stage::Ingestion,
            Stage::Deconstruction,
            Stage::CompatibilityGate,
            Stage::PainPointMining,
            Stage::EvolutionSimulation,
            Stage::Synthesis,
            Stage::Approval,
            Stage::Build,
        ]
    }

    pub fn next_stage(&self) -> Option<Stage> {
        let order = Self::stage_order();
        let idx = order.iter().position(|s| s == &self.current)?;
        order.get(idx + 1).copied()
    }

    /// Advance one stage if it is the expected next stage.
    pub fn advance(&mut self, stage: Stage) -> Result<(), CoreError> {
        if stage != self.current {
            return Err(CoreError::InvalidTargetCount(0)); // placeholder stage-order error
        }
        self.history.push(StageState {
            stage: self.current,
            completed: true,
            notes: format!("completed {:?}", self.current),
        });
        if let Some(next) = self.next_stage() {
            self.current = next;
        }
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        self.history.len() == Self::stage_order().len()
    }

    pub fn skips_gate_ok(&self, target_count: usize) -> bool {
        // Single-target skips compatibility gate evaluation (record as N/A).
        target_count == 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuiltArtifact {
    pub name: String,
    pub state_machine_nodes: usize,
    pub visual_tokens: VisualTokens,
    pub fidelity_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FidelityReport {
    pub node_match: bool,
    pub visual_match: bool,
    pub score_0_100: u8,
}

pub fn study_vs_build_verify(studied: &TargetDNA, built: &BuiltArtifact) -> FidelityReport {
    let node_match = studied.state_machine_nodes == built.state_machine_nodes;
    let visual_match = studied.visual_tokens.heading_font == built.visual_tokens.heading_font
        && studied.visual_tokens.body_font == built.visual_tokens.body_font
        && studied.visual_tokens.accent_hex == built.visual_tokens.accent_hex;
    let score = (node_match as u8) * 50 + (visual_match as u8) * 50;
    FidelityReport {
        node_match,
        visual_match,
        score_0_100: score,
    }
}

/// Static inventory of upstream `storytold/craft-anything` integration surface.
/// Live clone at `repos/craft-anything-upstream`.
pub fn craft_anything_inventory() -> serde_json::Value {
    serde_json::json!({
        "repo": "https://github.com/storytold/craft-anything.git",
        "claimed_tools": 314,
        "crates": ["ai","analysis","disasm","engine","engines","evidence","formats","js","mcp","platform","setup","testkit","ui-egui"],
        "study_modes": ["visible-only","deep-study"],
        "fidelity": "study-vs-build verification checks"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run_config_single_ok() {
        let c = RunConfig::new(
            RunMode::SingleUltimateEvolution,
            StudyMode::DeepStudy,
            vec![TargetSource::Url("https://example.com".into())],
        )
        .unwrap();
        assert_eq!(c.targets.len(), 1);
    }
    #[test]
    fn run_config_fusion_rejects_one() {
        assert!(RunConfig::new(
            RunMode::OmniverseFusion,
            StudyMode::DeepStudy,
            vec![TargetSource::Url("https://a.com".into())]
        )
        .is_err());
    }
    #[test]
    fn run_config_fusion_rejects_eleven() {
        let v: Vec<TargetSource> = (0..11)
            .map(|i| TargetSource::Url(format!("https://x{i}.com")))
            .collect();
        assert!(RunConfig::new(RunMode::OmniverseFusion, StudyMode::DeepStudy, v).is_err());
    }
    #[test]
    fn orchestrator_full_path() {
        let mut o = Orchestrator::default();
        for s in Orchestrator::stage_order() {
            assert_eq!(o.current, s);
            assert!(o.advance(s).is_ok());
        }
        assert!(o.is_complete());
    }
    #[test]
    fn fidelity_full_match() {
        let d = TargetDNA {
            name: "x".into(),
            source: TargetSource::Url("x".into()),
            core_loop: CoreLoop::AsyncCrud,
            input_topology: InputTopology::Pointer,
            domain_tags: vec!["x".into()],
            schema_fields: 5,
            concurrency_class: ConcurrencyClass::SoloLocal,
            monetization_loop: "saas".into(),
            telemetry: InteractionTelemetry {
                session_length_secs: 10.0,
                inputs_per_min: 10.0,
                concurrent_controls: 1,
            },
            visual_tokens: VisualTokens {
                heading_font: "Inter".into(),
                body_font: "Inter".into(),
                mono_font: "Mono".into(),
                accent_hex: "#112233".into(),
                radius_px: 4,
            },
            state_machine_nodes: 7,
        };
        let b = BuiltArtifact {
            name: "x".into(),
            state_machine_nodes: 7,
            visual_tokens: d.visual_tokens.clone(),
            fidelity_hash: "abc".into(),
        };
        let r = study_vs_build_verify(&d, &b);
        assert_eq!(r.score_0_100, 100);
    }
    #[test]
    fn inventory_lists_crates() {
        let inv = craft_anything_inventory();
        assert_eq!(inv["claimed_tools"], 314);
    }
}
