# ULTIMATRIX Pipeline — 9-Stage Deterministic State Machine

```
Ingestion → Deconstruction → Gate(≥2) → PainMining → EvolutionSim → Synthesis → Approval → Build
```

Single-target Mode A skips gate scoring (record N/A). Multi-target (2–10) hard-aborts at
Stage 3 when score <50 (cognitive veto when domain ≤20).

## Approval card (Stage 7)
`outputs/run.json` + `outputs/approval-card.json`:
gate score/verdict, bugs solved, MiroFish metrics
(task_velocity, churn_rate, cognitive_friction, viral_retention, edge_case_survival),
token telemetry (per-stage budgets + $).

## AGPL boundary
`MiroFish` (AGPL-3.0) is invoked only via `MiroFishClient::run_simulation_http`
(`POST /simulate`, `GET /health`). `dry_run=true` by default for offline/CI.
Never depend on AGPL code from the MIT/Apache crates.

## Study-vs-build fidelity
`study_vs_build_verify(studied DNA, built artifact)` → node + visual match → 0–100.
50 pts state-machine nodes, 50 pts design tokens (fonts/accent).
