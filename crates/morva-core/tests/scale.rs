//! NFR-04 scale evidence.
//!
//! The blocking assertions here are structural: parse and analysis output must grow in
//! exact proportion to input scale. Those counts are deterministic, so they are immune to
//! CI jitter and still fail loudly if any stage starts producing combinatorial output.
//!
//! Wall clock is used only as a single, deliberately generous absolute ceiling on the
//! largest scale. It is not a ratio between scales, because NFR-04 rules out a strict
//! wall-clock ratio as the first blocking gate. The margin below is large enough that a
//! genuinely quadratic regression fails it while ordinary CI noise cannot.

use morva_core::{Declaration, analyze, parse, simulate};
use std::time::{Duration, Instant};

/// One scale unit contributes one entity, one action, one scenario, and one compatibility
/// container, so every count below is a fixed multiple of the scale.
const BASE_SCALE: usize = 100;
const SCALES: [usize; 3] = [BASE_SCALE, BASE_SCALE * 4, BASE_SCALE * 16];

/// Generous enough to absorb a slow shared runner, small enough that quadratic growth at
/// the largest scale cannot hide under it.
const LARGEST_SCALE_CEILING: Duration = Duration::from_secs(30);

fn model(scale: usize) -> String {
    let mut source = String::from("system Scale {\n  enum Status {\n    Pending\n    Done\n  }\n");
    for index in 0..scale {
        source.push_str(&format!(
            "  entity E{index} {{\n    status: Status\n    count: Integer\n    invariant count >= 0\n  }}\n"
        ));
    }
    for index in 0..scale {
        source.push_str(&format!(
            "  action A{index}(item: E{index}) {{\n    idempotent by item.count\n    requires item.status == Pending\n    effects item.status = Done\n    ensures item.status == Done\n  }}\n"
        ));
    }
    for index in 0..scale {
        source.push_str(&format!(
            "  scenario S{index} {{\n    given item.status = Pending\n    given item.count = 0\n    run A{index}(item)\n    expect item.status == Done\n  }}\n"
        ));
    }
    for index in 0..scale {
        source.push_str(&format!(
            "  policy P{index} {{\n    unmodeled rule text {index}\n  }}\n"
        ));
    }
    source.push_str("}\n");
    source
}

fn declaration_counts(declarations: &[Declaration]) -> (usize, usize, usize, usize) {
    let mut entities = 0;
    let mut actions = 0;
    let mut scenarios = 0;
    let mut containers = 0;
    for declaration in declarations {
        match declaration {
            Declaration::Entity(_) => entities += 1,
            Declaration::Action(_) => actions += 1,
            Declaration::Scenario(_) => scenarios += 1,
            Declaration::Container(_) => containers += 1,
            _ => {}
        }
    }
    (entities, actions, scenarios, containers)
}

#[test]
fn parsed_declarations_grow_in_exact_proportion_to_model_scale() {
    for scale in SCALES {
        let document = parse(&model(scale)).expect("generated scale model parses");
        let system = document
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::System(system) => Some(system),
                _ => None,
            })
            .expect("generated model has one top-level system");
        let counts = declaration_counts(&system.declarations);
        assert_eq!(
            counts,
            (scale, scale, scale, scale),
            "scale {scale} must produce one entity, action, scenario, and container per unit"
        );
    }
}

#[test]
fn analysis_findings_grow_in_exact_proportion_to_model_scale() {
    for scale in SCALES {
        let document = parse(&model(scale)).expect("generated scale model parses");
        let report = analyze(&document);
        assert!(
            report.errors.is_empty(),
            "scale {scale} model must be semantically clean, got {:?}",
            report.errors.first()
        );
        // One MORVA5001 per policy container plus one MORVA5002 per `idempotent` item.
        assert_eq!(
            report.notices.len(),
            scale * 2,
            "scale {scale} must produce exactly two notices per unit"
        );
        assert_eq!(
            report.findings().len(),
            scale * 2,
            "merged findings must not duplicate or drop notices at scale {scale}"
        );
    }
}

#[test]
fn largest_scale_model_parses_analyzes_and_simulates_within_an_absolute_ceiling() {
    let scale = *SCALES.last().expect("scale list is non-empty");
    let source = model(scale);

    let started = Instant::now();
    let document = parse(&source).expect("largest scale model parses");
    let report = analyze(&document);
    let simulation =
        simulate(&document, &format!("S{}", scale - 1)).expect("last scenario simulates");
    let elapsed = started.elapsed();

    assert!(
        report.errors.is_empty(),
        "largest scale model must be clean"
    );
    assert!(
        simulation.phases.iter().all(|phase| phase.passed),
        "every phase of the last scenario must pass"
    );
    assert_eq!(
        simulation.phases.len(),
        7,
        "simulation must still report the fixed phase sequence at scale"
    );
    assert!(
        elapsed < LARGEST_SCALE_CEILING,
        "scale {scale} took {elapsed:?}, above the {LARGEST_SCALE_CEILING:?} ceiling; \
         investigate super-linear growth before raising this bound"
    );
}
