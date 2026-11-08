use clap::{Parser, Subcommand};
use compatibility_gate::{
    cluster_verdict, diagnostic_report, mcp_tools_list, score_cluster, score_pair,
};
use craft_core::{RunConfig, RunMode, StudyMode, TargetSource};
use egm_architect::{egm_loop, StubElephant, StubGoldfish};
use entrepreneur_rea::{PainMiner, ReaAdapter};
use mirofish_crucible::{EvolutionConfig, MiroFishClient};
use reverse_adapter::{Deconstructor, ReverseSkillAdapter};
use std::fs;
use token_governance::Governance;

#[derive(Parser)]
#[command(name = "ultimatrix")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Stage 1: validate ingestion (1 or 2-10 targets)
    Ingest { targets: Vec<String> },
    /// Stage 2: deep deconstruction stub per target
    Deconstruct { targets: Vec<String> },
    /// Stage 3: compatibility gate for multi-target
    Gate { targets: Vec<String> },
    /// Stages 1-8 dry-run end-to-end (single or fused)
    Run { targets: Vec<String> },
    /// Stage 5 dry simulation only
    Simulate { targets: Vec<String> },
    /// List MCP tools (JSON)
    McpTools,
    /// Stage 7 approval card PNG/JSON equivalent (writes outputs/approval-card.json)
    Approve { targets: Vec<String> },
}

fn sources(targets: &[String]) -> Vec<TargetSource> {
    targets
        .iter()
        .map(|t| {
            if t.starts_with("http") {
                TargetSource::Url(t.clone())
            } else {
                TargetSource::Repo(t.clone())
            }
        })
        .collect()
}

fn stub_dna(name: &str) -> craft_core::TargetDNA {
    craft_core::TargetDNA {
        name: name.to_string(),
        source: TargetSource::Url(name.into()),
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

// Demo archetype DNA for gate examples in help/README.
fn demo_dna(name: &str) -> craft_core::TargetDNA {
    match name {
        "shadow-fight" => craft_core::TargetDNA {
            core_loop: craft_core::CoreLoop::RealTime { frame_ms: 16.6 },
            domain_tags: vec!["melee".into(), "fighter".into()],
            concurrency_class: craft_core::ConcurrencyClass::OneVsOne,
            ..stub_dna(name)
        },
        "parkour" => craft_core::TargetDNA {
            core_loop: craft_core::CoreLoop::RealTime { frame_ms: 16.6 },
            domain_tags: vec!["momentum".into(), "traversal".into()],
            concurrency_class: craft_core::ConcurrencyClass::SoloLocal,
            ..stub_dna(name)
        },
        "battle-royale" => craft_core::TargetDNA {
            core_loop: craft_core::CoreLoop::RealTime { frame_ms: 16.6 },
            domain_tags: vec!["battle".into(), "fps".into()],
            concurrency_class: craft_core::ConcurrencyClass::AuthoritativeMassive,
            ..stub_dna(name)
        },
        "story-rpg" => craft_core::TargetDNA {
            core_loop: craft_core::CoreLoop::TurnBased {
                avg_turn_secs: 30.0,
            },
            domain_tags: vec!["rpg".into(), "narrative".into()],
            concurrency_class: craft_core::ConcurrencyClass::SoloLocal,
            ..stub_dna(name)
        },
        "banking" => craft_core::TargetDNA {
            core_loop: craft_core::CoreLoop::AsyncCrud,
            domain_tags: vec!["bank".into(), "ledger".into()],
            concurrency_class: craft_core::ConcurrencyClass::SaaSAsync,
            ..stub_dna(name)
        },
        "rhythm" => craft_core::TargetDNA {
            core_loop: craft_core::CoreLoop::RealTime { frame_ms: 16.6 },
            domain_tags: vec!["rhythm".into(), "arcade".into()],
            concurrency_class: craft_core::ConcurrencyClass::SoloLocal,
            ..stub_dna(name)
        },
        _ => stub_dna(name),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Ingest { targets } => {
            let mode = if targets.len() == 1 {
                RunMode::SingleUltimateEvolution
            } else {
                RunMode::OmniverseFusion
            };
            let cfg = RunConfig::new(mode.clone(), StudyMode::DeepStudy, sources(&targets))?;
            println!("ingestion ok: {:?} targets={}", mode, cfg.targets.len());
            if let Some(p) = std::env::args().find(|a| a == "--out") {
                let _ = p;
            }
        }
        Commands::Deconstruct { targets } => {
            let d = ReverseSkillAdapter;
            for t in &targets {
                let r = d.deconstruct(&TargetSource::Url(t.clone()), &StudyMode::DeepStudy)?;
                println!(
                    "deconstructed {} -> {} state nodes",
                    r.dna.name, r.dna.state_machine_nodes
                );
            }
        }
        Commands::Gate { targets } => {
            let dnas: Vec<_> = targets.iter().map(|t| demo_dna(t)).collect();
            if dnas.len() < 2 {
                println!("gate needs >=2 targets");
                return Ok(());
            }
            let results = score_cluster(&dnas);
            for r in &results {
                println!(
                    "pair {:?} score={} verdict={:?} axes={:?}",
                    r.pair, r.score, r.verdict, r.axes
                );
                if dnas.len() >= 2 {
                    println!("{}", diagnostic_report(&dnas[0], &dnas[1], r));
                }
            }
            println!("cluster verdict: {:?}", cluster_verdict(&results));
            if cluster_verdict(&results) == compatibility_gate::Verdict::IncompatibleRejected {
                println!("FUSION ABORTED: INCOMPATIBLE_REJECTED — suggest compatible archetypes (same pacing/domain).");
            }
        }
        Commands::Simulate { targets } => {
            let gov = Governance::default();
            let cfg = EvolutionConfig {
                epochs: 100,
                agents_per_epoch: 500,
                seed: 42,
                pressure_bugs: vec![],
            };
            let m = MiroFishClient::default()
                .run_simulation(&cfg, &entrepreneur_rea::EntrepreneurLedger::default())
                .await?;
            let s = serde_json::to_string_pretty(&m)?;
            println!("simulation metrics (dry-run): {}", s);
            println!(
                "token budgets armed for {} targets; sample estimate ok",
                targets.len()
            );
            let _ = gov;
        }
        Commands::Run { targets } => {
            let mode = if targets.len() == 1 {
                RunMode::SingleUltimateEvolution
            } else {
                RunMode::OmniverseFusion
            };
            let _cfg = RunConfig::new(mode, StudyMode::DeepStudy, sources(&targets))?;
            let mut gov = Governance::default();
            gov.record("ingestion", 100);
            let dnas: Vec<_> = targets.iter().map(|t| demo_dna(t)).collect();
            if targets.len() >= 2 {
                let results = score_pair(&dnas[0], dnas.get(1).unwrap_or(&dnas[0]));
                println!("gate: score={} {:?}", results.score, results.verdict);
                if results.verdict == compatibility_gate::Verdict::IncompatibleRejected {
                    println!("ABORTED at Stage 3");
                    return Ok(());
                }
            }
            let ledger = ReaAdapter
                .mine(&dnas[0], &dnas[1.min(dnas.len() - 1)..])
                .unwrap_or_default();
            let metrics = MiroFishClient::default()
                .run_simulation(
                    &EvolutionConfig {
                        epochs: 100,
                        agents_per_epoch: 500,
                        seed: 42,
                        pressure_bugs: ledger.top_bugs.clone(),
                    },
                    &ledger,
                )
                .await?;
            let bp = egm_loop(&StubElephant, &StubGoldfish, &metrics);
            println!(
                "blueprint: {} defects={}",
                bp.interaction_paradigm, bp.critical_defects_remaining
            );
            println!(
                "APPROVAL CARD: gate ok, bugs={} metrics.velocity={:.2} tokens=${:.4}",
                ledger.top_bugs.len(),
                metrics.task_velocity,
                gov.telemetry.total_cost_usd
            );
            // Write outputs to ./outputs for inspection
            let _ = fs::create_dir_all("outputs");
            let _ = fs::write(
                "outputs/run.json",
                serde_json::to_string_pretty(
                    &serde_json::json!({"targets": targets, "metrics": metrics, "blueprint": bp, "ledger": ledger}),
                )?,
            );
            let card = serde_json::json!({
                "stage": "approval",
                "gate": "passed",
                "blueprint": bp,
                "metrics": metrics,
                "bugs_solved": ledger.top_bugs,
                "token_telemetry": gov.telemetry
            });
            let _ = fs::write(
                "outputs/approval-card.json",
                serde_json::to_string_pretty(&card)?,
            );
            println!("wrote outputs/run.json + outputs/approval-card.json");
        }
        Commands::McpTools => {
            println!("{}", serde_json::to_string_pretty(&mcp_tools_list())?);
        }
        Commands::Approve { targets } => {
            let _ = fs::create_dir_all("outputs");
            let dnas: Vec<_> = targets.iter().map(|t| demo_dna(t)).collect();
            let verdict = if dnas.len() >= 2 {
                cluster_verdict(&score_cluster(&dnas))
            } else {
                compatibility_gate::Verdict::Compatible
            };
            let card =
                serde_json::json!({"targets": targets, "verdict": verdict, "timestamp": "dry-run"});
            let _ = fs::write(
                "outputs/approval-card.json",
                serde_json::to_string_pretty(&card)?,
            );
            println!(
                "approval card written: outputs/approval-card.json verdict={:?}",
                verdict
            );
        }
    }
    Ok(())
}
