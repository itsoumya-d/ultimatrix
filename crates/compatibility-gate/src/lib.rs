#[cfg(test)]
use craft_core::ConcurrencyClass;
use craft_core::{CoreLoop, InputTopology, TargetDNA};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Verdict {
    Compatible,
    ConditionalHybrid,
    IncompatibleRejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AxisScores {
    pub loop_resonance: u8,
    pub input_topology: u8,
    pub domain_coherence: u8,
    pub composability: u8,
    pub scale_compat: u8,
    pub commercial_harmony: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateResult {
    pub score: u8,
    pub axes: AxisScores,
    pub verdict: Verdict,
    pub pair: (String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FusionBridgeSpec {
    pub bridge_loop: String,
    pub shared_inputs: Vec<String>,
    pub shared_state: Vec<String>,
    pub stripped_features: Vec<String>,
}

fn clamp(v: f32) -> u8 {
    v.round().clamp(0.0, 100.0) as u8
}

fn loop_component(a: &TargetDNA, b: &TargetDNA) -> u8 {
    match (&a.core_loop, &b.core_loop) {
        (CoreLoop::RealTime { frame_ms: fa }, CoreLoop::RealTime { frame_ms: fb }) => {
            let d = (fa - fb).abs();
            if d < 8.0 {
                90
            } else if d < 16.0 {
                75
            } else if d < 33.0 {
                55
            } else {
                30
            }
        }
        (CoreLoop::RealTime { .. }, CoreLoop::TurnBased { .. })
        | (CoreLoop::TurnBased { .. }, CoreLoop::RealTime { .. })
        | (CoreLoop::RealTime { .. }, CoreLoop::AsyncCrud)
        | (CoreLoop::AsyncCrud, CoreLoop::RealTime { .. }) => 20,
        (CoreLoop::AsyncCrud, CoreLoop::AsyncCrud) => 85,
        (CoreLoop::TurnBased { .. }, CoreLoop::TurnBased { .. }) => 80,
        (CoreLoop::Mixed { .. }, _) | (_, CoreLoop::Mixed { .. }) => 60,
        _ => 50,
    }
}

fn input_component(a: &TargetDNA, b: &TargetDNA) -> u8 {
    let ca = concurrent(a);
    let cb = concurrent(b);
    let total = ca + cb;
    if total <= 6 {
        90
    } else if total <= 9 {
        70
    } else if total <= 12 {
        55
    } else {
        25
    }
}

fn concurrent(d: &TargetDNA) -> u32 {
    let mut base = d.telemetry.concurrent_controls;
    // TextHeavy alone is low ergonomic load, RealTime+TextHeavy stacks.
    match &d.input_topology {
        InputTopology::TextHeavy => base = base.min(2),
        InputTopology::Keybinds | InputTopology::TouchGestures => {}
        _ => {}
    }
    base
}

fn domain_component(a: &TargetDNA, b: &TargetDNA) -> u8 {
    let sa: std::collections::HashSet<&String> = a.domain_tags.iter().collect();
    let sb: std::collections::HashSet<&String> = b.domain_tags.iter().collect();
    let inter = sa.intersection(&sb).count() as f32;
    let union = (sa.len() + sb.len()) as f32 - inter;
    let jaccard = if union == 0.0 { 1.0 } else { inter / union };
    // Adjacency pairs known from spec: code editor + terminal + debugger = high.
    let joined = format!("{:?}|{:?}", a.domain_tags, b.domain_tags).to_lowercase();
    if joined.contains("code") && joined.contains("terminal") {
        return 90;
    }
    if joined.contains("bank") && joined.contains("rhythm") {
        return 10;
    }
    if joined.contains("fighter") || joined.contains("parkour") || joined.contains("melee") {
        return 85;
    }
    if joined.contains("battle") && joined.contains("rpg") {
        return 20;
    }
    clamp(jaccard * 100.0 + 20.0)
}

fn composability_component(a: &TargetDNA, b: &TargetDNA) -> u8 {
    let d = (a.schema_fields as f32 - b.schema_fields as f32).abs();
    if d <= 10.0 {
        90
    } else if d <= 40.0 {
        70
    } else {
        40
    }
}

fn scale_component(a: &TargetDNA, b: &TargetDNA) -> u8 {
    let diff = (a.concurrency_class as i32 - b.concurrency_class as i32).abs();
    match diff {
        0 => 90,
        1 => 75,
        2 => 55,
        _ => 25,
    }
}

fn commercial_component(a: &TargetDNA, b: &TargetDNA) -> u8 {
    if a.monetization_loop == b.monetization_loop {
        85
    } else if a.monetization_loop.contains("saas") || b.monetization_loop.contains("saas") {
        70
    } else if a.monetization_loop.is_empty() || b.monetization_loop.is_empty() {
        60
    } else {
        50
    }
}

pub fn score_pair(a: &TargetDNA, b: &TargetDNA) -> GateResult {
    let axes = AxisScores {
        loop_resonance: loop_component(a, b),
        input_topology: input_component(a, b),
        domain_coherence: domain_component(a, b),
        composability: composability_component(a, b),
        scale_compat: scale_component(a, b),
        commercial_harmony: commercial_component(a, b),
    };
    let w = (axes.loop_resonance as f32 * 25.0
        + axes.input_topology as f32 * 20.0
        + axes.domain_coherence as f32 * 20.0
        + axes.composability as f32 * 15.0
        + axes.scale_compat as f32 * 10.0
        + axes.commercial_harmony as f32 * 10.0)
        / 100.0;
    let mut score = clamp(w);
    // Hard cognitive veto: incoherent domains cannot be saved by other axes.
    if axes.domain_coherence <= 20 {
        score = score.min(40);
    }
    let verdict = if score >= 75 {
        Verdict::Compatible
    } else if score >= 50 {
        Verdict::ConditionalHybrid
    } else {
        Verdict::IncompatibleRejected
    };
    GateResult {
        score,
        axes,
        verdict,
        pair: (a.name.clone(), b.name.clone()),
    }
}

pub fn score_cluster(targets: &[TargetDNA]) -> Vec<GateResult> {
    let mut out = Vec::new();
    for i in 0..targets.len() {
        for j in (i + 1)..targets.len() {
            out.push(score_pair(&targets[i], &targets[j]));
        }
    }
    out
}

pub fn cluster_verdict(results: &[GateResult]) -> Verdict {
    let min = results.iter().map(|r| r.score).min().unwrap_or(100);
    if min >= 75 {
        Verdict::Compatible
    } else if min >= 50 {
        Verdict::ConditionalHybrid
    } else {
        Verdict::IncompatibleRejected
    }
}

pub fn fusion_bridge(a: &TargetDNA, b: &TargetDNA, verdict: &Verdict) -> FusionBridgeSpec {
    let bridge_loop = match verdict {
        Verdict::Compatible => format!("Shared real-time loop: {} + {}", a.name, b.name),
        Verdict::ConditionalHybrid => format!(
            "Stripped core: {} core + {} core (sub-features removed)",
            a.name, b.name
        ),
        Verdict::IncompatibleRejected => "NO BRIDGE — fusion aborted".into(),
    };
    FusionBridgeSpec {
        bridge_loop,
        shared_inputs: vec![
            format!("{:?}", a.input_topology),
            format!("{:?}", b.input_topology),
        ],
        shared_state: vec!["entity_graph".into(), "session_telemetry".into()],
        stripped_features: if *verdict == Verdict::ConditionalHybrid {
            vec![
                "conflicting_inputs".into(),
                "incompatible_pacing_windows".into(),
            ]
        } else {
            vec![]
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalCard {
    pub verdict: Verdict,
    pub score: u8,
    pub pair: (String, String),
    pub top_bugs_solved: Vec<String>,
    pub metrics_summary: String,
    pub token_telemetry: String,
}

pub fn diagnostic_report(a: &TargetDNA, b: &TargetDNA, r: &GateResult) -> String {
    let mut s = format!(
        "GATE DIAGNOSTIC: {} + {} score={} verdict={:?}\n",
        a.name, b.name, r.score, r.verdict
    );
    s.push_str(&format!(
        "axes: loop={} input={} domain={} composability={} scale={} commercial={}\n",
        r.axes.loop_resonance,
        r.axes.input_topology,
        r.axes.domain_coherence,
        r.axes.composability,
        r.axes.scale_compat,
        r.axes.commercial_harmony
    ));
    if r.axes.loop_resonance < 50 {
        s.push_str("- FAIL loop: incompatible pacing/time scale; do not fuse real-time with turn-based/async without adaptation layer.\n");
    }
    if r.axes.domain_coherence <= 20 {
        s.push_str(
            "- FAIL domain: cognitive incoherence; user intents in incompatible mental states.\n",
        );
    }
    if r.axes.input_topology < 50 {
        s.push_str("- FAIL input: combined concurrent controls exceed ergonomic budget.\n");
    }
    if r.axes.composability < 50 {
        s.push_str("- FAIL composability: schema impedance / circular state risk.\n");
    }
    if r.axes.scale_compat < 50 {
        s.push_str("- FAIL scale: concurrency topologies diverge (e.g. 1v1 local vs authoritative massive).\n");
    }
    if r.verdict == Verdict::IncompatibleRejected {
        s.push_str("HARD ABORT. Suggested archetypes: same pacing class + adjacent domain + same concurrency tier.\n");
    } else if r.verdict == Verdict::ConditionalHybrid {
        s.push_str("CONDITIONAL: strip listed sub-features, fuse harmonious core only.\n");
    } else {
        s.push_str("COMPATIBLE: emit FusionBridgeSpec.\n");
    }
    s
}

pub fn mcp_tools_list() -> serde_json::Value {
    serde_json::json!([
        {"name":"ultimatrix.ingest","description":"Stage 1 validate 1 or 2-10 targets"},
        {"name":"ultimatrix.deconstruct","description":"Stage 2 reverse/adaptation to TargetDNA"},
        {"name":"ultimatrix.gate","description":"Stage 3 6-axis compatibility gate"},
        {"name":"ultimatrix.mine","description":"Stage 4 entrepreneur pain-point ledger"},
        {"name":"ultimatrix.simulate","description":"Stage 5 MiroFish HTTP crucible (dry-run default)"},
        {"name":"ultimatrix.synthesize","description":"Stage 6 EGM Elephant/Goldfish loop"},
        {"name":"ultimatrix.approve","description":"Stage 7 approval card"},
        {"name":"ultimatrix.build","description":"Stage 8 clean-room build stub"}
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_core::{InteractionTelemetry, TargetSource, VisualTokens};

    fn dna(
        name: &str,
        loop_: CoreLoop,
        tags: &[&str],
        conc: u32,
        scale: ConcurrencyClass,
    ) -> TargetDNA {
        TargetDNA {
            name: name.into(),
            source: TargetSource::Url(name.into()),
            core_loop: loop_,
            input_topology: InputTopology::Keybinds,
            domain_tags: tags.iter().map(|s| s.to_string()).collect(),
            schema_fields: 10,
            concurrency_class: scale,
            monetization_loop: "saas".into(),
            telemetry: InteractionTelemetry {
                session_length_secs: 300.0,
                inputs_per_min: 60.0,
                concurrent_controls: conc,
            },
            visual_tokens: VisualTokens {
                heading_font: "Inter".into(),
                body_font: "Inter".into(),
                mono_font: "JetBrainsMono".into(),
                accent_hex: "#000000".into(),
                radius_px: 8,
            },
            state_machine_nodes: 20,
        }
    }

    fn shadow_fight() -> TargetDNA {
        dna(
            "Shadow Fight",
            CoreLoop::RealTime { frame_ms: 16.6 },
            &["melee", "fighter"],
            3,
            ConcurrencyClass::OneVsOne,
        )
    }
    fn parkour() -> TargetDNA {
        dna(
            "Parkour",
            CoreLoop::RealTime { frame_ms: 16.6 },
            &["momentum", "traversal"],
            3,
            ConcurrencyClass::SoloLocal,
        )
    }
    fn battle_royale() -> TargetDNA {
        dna(
            "Battle Royale FPS",
            CoreLoop::RealTime { frame_ms: 16.6 },
            &["battle", "royale", "fps"],
            4,
            ConcurrencyClass::AuthoritativeMassive,
        )
    }
    fn story_rpg() -> TargetDNA {
        dna(
            "Story RPG",
            CoreLoop::TurnBased {
                avg_turn_secs: 30.0,
            },
            &["rpg", "narrative"],
            1,
            ConcurrencyClass::SoloLocal,
        )
    }
    fn banking() -> TargetDNA {
        dna(
            "Banking",
            CoreLoop::AsyncCrud,
            &["bank", "ledger"],
            1,
            ConcurrencyClass::SaaSAsync,
        )
    }
    fn rhythm() -> TargetDNA {
        dna(
            "Rhythm",
            CoreLoop::RealTime { frame_ms: 16.6 },
            &["rhythm", "arcade"],
            2,
            ConcurrencyClass::SoloLocal,
        )
    }
    fn code_editor() -> TargetDNA {
        dna(
            "Code Editor",
            CoreLoop::AsyncCrud,
            &["code", "editor"],
            2,
            ConcurrencyClass::SaaSAsync,
        )
    }
    fn terminal() -> TargetDNA {
        dna(
            "Terminal",
            CoreLoop::AsyncCrud,
            &["terminal", "shell"],
            2,
            ConcurrencyClass::SaaSAsync,
        )
    }

    #[test]
    fn shadow_parkour_compatible() {
        let r = score_pair(&shadow_fight(), &parkour());
        assert!(r.score >= 50, "got {}", r.score);
        assert_ne!(r.verdict, Verdict::IncompatibleRejected);
    }
    #[test]
    fn br_vs_rpg_rejected() {
        let r = score_pair(&battle_royale(), &story_rpg());
        assert!(r.score < 50, "got {}", r.score);
    }
    #[test]
    fn banking_vs_rhythm_rejected() {
        let r = score_pair(&banking(), &rhythm());
        assert!(r.score < 50, "got {}", r.score);
    }
    #[test]
    fn code_terminal_high() {
        let r = score_pair(&code_editor(), &terminal());
        assert!(r.score >= 50, "got {}", r.score);
    }
    #[test]
    fn cluster_min_verdict() {
        let v = score_cluster(&[shadow_fight(), parkour()]);
        assert_eq!(
            cluster_verdict(&v),
            score_pair(&shadow_fight(), &parkour()).verdict
        );
    }
}
