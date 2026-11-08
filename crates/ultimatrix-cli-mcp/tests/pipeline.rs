// Integration: gate + orchestrator + fidelity end-to-end (dry).
use craft_core::*;
use compatibility_gate::*;

#[test]
fn demo_fusion_pair_passes() {
    let make = |name: &str| TargetDNA {
        name: name.into(),
        source: TargetSource::Url(name.into()),
        core_loop: CoreLoop::RealTime { frame_ms: 16.6 },
        input_topology: InputTopology::Keybinds,
        domain_tags: vec!["melee".into()],
        schema_fields: 10,
        concurrency_class: ConcurrencyClass::OneVsOne,
        monetization_loop: "saas".into(),
        telemetry: InteractionTelemetry { session_length_secs: 100.0, inputs_per_min: 50.0, concurrent_controls: 2 },
        visual_tokens: VisualTokens { heading_font: "Inter".into(), body_font: "Inter".into(), mono_font: "Mono".into(), accent_hex: "#000000".into(), radius_px: 8 },
        state_machine_nodes: 20,
    };
    let r = score_pair(&make("shadow-fight"), &make("parkour"));
    assert_eq!(r.verdict, Verdict::Compatible);
    let rep = diagnostic_report(&make("shadow-fight"), &make("parkour"), &r);
    assert!(rep.contains("COMPATIBLE"));
}

#[test]
fn orchestrator_advances_all_stages() {
    let mut o = Orchestrator::default();
    for s in Orchestrator::stage_order() { assert!(o.advance(s).is_ok()); }
    assert!(o.is_complete());
}

#[test]
fn inventory_present() {
    assert_eq!(craft_core::craft_anything_inventory()["claimed_tools"], 314);
}
