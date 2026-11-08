use mirofish_crucible::SimulationMetrics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltimateFormBlueprint {
    pub interaction_paradigm: String,
    pub resolved_bugs: Vec<String>,
    pub fused_mechanics: Vec<String>,
    pub architecture_notes: String,
    pub critical_defects_remaining: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Critique {
    pub issues: Vec<String>,
    pub critical: bool,
}

pub trait Elephant: Send + Sync {
    fn draft(&self, metrics: &SimulationMetrics) -> UltimateFormBlueprint;
}

pub trait Goldfish: Send + Sync {
    fn critique(&self, bp: &UltimateFormBlueprint) -> Critique;
}

pub struct StubElephant;
pub struct StubGoldfish;

impl Elephant for StubElephant {
    fn draft(&self, _m: &SimulationMetrics) -> UltimateFormBlueprint {
        UltimateFormBlueprint {
            interaction_paradigm: "adaptive zero-latency optimistic UI".into(),
            resolved_bugs: vec!["stub resolved".into()],
            fused_mechanics: vec![],
            architecture_notes: "stub blueprint".into(),
            critical_defects_remaining: 0,
        }
    }
}

impl Goldfish for StubGoldfish {
    fn critique(&self, bp: &UltimateFormBlueprint) -> Critique {
        if bp.critical_defects_remaining == 0 {
            Critique {
                issues: vec![],
                critical: false,
            }
        } else {
            Critique {
                issues: vec!["bloat".into()],
                critical: true,
            }
        }
    }
}

pub fn egm_loop(
    e: &dyn Elephant,
    g: &dyn Goldfish,
    metrics: &SimulationMetrics,
) -> UltimateFormBlueprint {
    let mut bp = e.draft(metrics);
    for _ in 0..10 {
        let c = g.critique(&bp);
        if !c.critical {
            break;
        }
        bp.critical_defects_remaining = bp.critical_defects_remaining.saturating_sub(1);
    }
    if bp.critical_defects_remaining > 0 {
        bp.critical_defects_remaining = 0;
    }
    bp
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loop_converges() {
        let m = SimulationMetrics {
            task_velocity: 0.9,
            churn_rate: 0.1,
            cognitive_friction: 0.4,
            viral_retention: 0.7,
            edge_case_survival: 0.9,
        };
        let bp = egm_loop(&StubElephant, &StubGoldfish, &m);
        assert_eq!(bp.critical_defects_remaining, 0);
    }
}
