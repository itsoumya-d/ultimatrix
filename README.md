# ULTIMATRIX — Pure-Rust Evolution & Fusion Engine

Redesign of `storytold/craft-anything` (314 tools, study-vs-build) into
an evolutionary crucible for 1–10 targets. Upstream clone preserved at
`../repos/craft-anything-upstream` (read-only reference).

## Workspace (8 crates)
- `craft-core` — `TargetDNA`, `RunConfig` (1 vs 2–10 validation), study/build traits
- `compatibility-gate` — 6 axes 25/20/20/15/10/10, hard abort <50, cognitive veto ≤20
- `reverse-adapter` — `reverse-skill` / `replica-skill` bridge (clean-room only)
- `entrepreneur-rea` — `morluto/rea` + review scrapers → `EntrepreneurLedger`
- `mirofish-crucible` — **isolated HTTP boundary only** for AGPL `MiroFish`; never linked
- `egm-architect` — Elephant draft + Goldfish blind critics until 0 critical
- `token-governance` — LLMLingua compress stub, Guidance JSON lock, per-stage budgets
- `ultimatrix-cli-mcp` — `ultimatrix` binary: ingest/deconstruct/gate/simulate/run

## 9-stage pipeline
1 Ingestion (1 or 2–10) → 2 Deconstruction → 3 Gate (mandatory ≥2) →
4 Pain mining → 5 MiroFish swarm (metrics: velocity/churn/friction/retention/survival)
→ 6 EGM → 7 Approval card → 8 Build (craft-core + replica-skill)

## Run
```bash
cargo test
cargo run -p ultimatrix-cli-mcp -- gate shadow-fight parkour        # 87 Compatible
cargo run -p ultimatrix-cli-mcp -- gate battle-royale story-rpg    # 40 REJECTED
cargo run -p ultimatrix-cli-mcp -- gate banking rhythm             # 40 REJECTED
cargo run -p ultimatrix-cli-mcp -- run shadow-fight parkour        # full dry-run -> outputs/run.json
cargo run -p ultimatrix-cli-mcp -- run https://example.com         # Mode A single
```

## Boundaries & safety
- Clean-room only; no proprietary source copying.
- MiroFish (AGPL-3.0) over HTTP/gRPC, never compiled in.
- Production adapters (scrapers, live HTTP, GPT synthesis) are stubbed with
  deterministic, tested seams — wire real endpoints via config.
