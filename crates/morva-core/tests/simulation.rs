use morva_core::{SimulationPhase, Value, check, parse, simulate};

fn checked(source: &str) -> morva_core::Document {
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
    document
}

fn diagnostic_codes(source: &str) -> Vec<&'static str> {
    let document = parse(source).expect("valid syntax");
    check(&document).into_iter().map(|item| item.code).collect()
}

#[test]
fn simulates_the_repository_example() {
    let document = checked(include_str!("../../../examples/order.morva"));
    let report = simulate(&document, "NormalConfirmation").expect("scenario selected");
    assert!(report.succeeded());
    assert_eq!(report.action, "Confirm");
    assert_eq!(report.phases.len(), 7);
    assert!(report.phases.iter().all(|phase| phase.passed));
    assert_eq!(report.changes.len(), 1);
    assert_eq!(report.changes[0].path, "order.status");
    assert_eq!(
        report.changes[0].before.as_ref().unwrap().to_string(),
        "Pending"
    );
    assert_eq!(report.changes[0].after.to_string(), "Confirmed");
}

#[test]
fn initial_invariant_failure_stops_before_effects() {
    let source = r#"system Test {
  entity Counter {
    count: Integer
    invariant count > 0
  }
  action Increment(counter: Counter) { effects counter.count += 1 }
  scenario Invalid {
    given counter.count = 0
    run Increment(counter)
    expect true
  }
}
"#;
    let report = simulate(&checked(source), "Invalid").unwrap();
    assert_eq!(
        report.failure.as_ref().unwrap().phase,
        SimulationPhase::InitialInvariants
    );
    assert!(report.changes.is_empty());
    assert_eq!(report.state["counter.count"], Value::Integer(0));
}

#[test]
fn duplicate_given_fails_in_the_givens_phase() {
    let source = r#"system Test {
  entity Counter { count: Integer }
  action Read(counter: Counter) {}
  scenario Duplicate {
    given counter.count = 1
    given counter.count = 2
    run Read(counter)
    expect true
  }
}
"#;
    let report = simulate(&checked(source), "Duplicate").unwrap();
    let failure = report.failure.unwrap();
    assert_eq!(failure.phase, SimulationPhase::Givens);
    assert!(failure.message.contains("duplicate initialization"));
}

#[test]
fn requires_failure_stops_before_effects() {
    let source = r#"system Test {
  entity Counter { count: Integer }
  action Increment(counter: Counter) {
    requires counter.count > 0
    effects counter.count += 1
  }
  scenario Invalid {
    given counter.count = 0
    run Increment(counter)
    expect true
  }
}
"#;
    let report = simulate(&checked(source), "Invalid").unwrap();
    assert_eq!(
        report.failure.as_ref().unwrap().phase,
        SimulationPhase::Requires
    );
    assert!(report.changes.is_empty());
}

#[test]
fn final_invariant_failure_preserves_effect_changes() {
    let source = r#"system Test {
  entity Counter {
    count: Integer
    invariant count >= 0
  }
  action Decrement(counter: Counter) { effects counter.count -= 2 }
  scenario Invalid {
    given counter.count = 1
    run Decrement(counter)
    expect true
  }
}
"#;
    let report = simulate(&checked(source), "Invalid").unwrap();
    assert_eq!(
        report.failure.as_ref().unwrap().phase,
        SimulationPhase::FinalInvariants
    );
    assert_eq!(report.changes.len(), 1);
    assert_eq!(report.state["counter.count"], Value::Integer(-1));
}

#[test]
fn ensures_and_expect_fail_in_their_own_phases() {
    let ensures = r#"system Test {
  entity Counter { count: Integer }
  action Increment(counter: Counter) {
    effects counter.count += 1
    ensures counter.count == 0
  }
  scenario Invalid {
    given counter.count = 0
    run Increment(counter)
    expect true
  }
}
"#;
    let report = simulate(&checked(ensures), "Invalid").unwrap();
    assert_eq!(
        report.failure.as_ref().unwrap().phase,
        SimulationPhase::Ensures
    );
    assert_eq!(report.state["counter.count"], Value::Integer(1));

    let expects = r#"system Test {
  entity Counter { count: Integer }
  action Increment(counter: Counter) { effects counter.count += 1 }
  scenario Invalid {
    given counter.count = 0
    run Increment(counter)
    expect counter.count == 0
  }
}
"#;
    let report = simulate(&checked(expects), "Invalid").unwrap();
    assert_eq!(
        report.failure.as_ref().unwrap().phase,
        SimulationPhase::Expects
    );
}

#[test]
fn uninitialized_read_is_a_stable_runtime_failure() {
    let source = r#"system Test {
  entity Counter { count: Integer }
  action Read(counter: Counter) { requires counter.count > 0 }
  scenario Missing {
    run Read(counter)
    expect true
  }
}
"#;
    let report = simulate(&checked(source), "Missing").unwrap();
    let failure = report.failure.unwrap();
    assert_eq!(failure.phase, SimulationPhase::Requires);
    assert!(failure.message.contains("uninitialized read"));
}

#[test]
fn compound_integer_overflow_fails_without_panicking() {
    let source = r#"system Test {
  entity Counter { count: Integer }
  action Increment(counter: Counter) { effects counter.count += 1 }
  scenario Overflow {
    given counter.count = 9223372036854775807
    run Increment(counter)
    expect true
  }
}
"#;
    let report = simulate(&checked(source), "Overflow").unwrap();
    let failure = report.failure.unwrap();
    assert_eq!(failure.phase, SimulationPhase::Effects);
    assert!(failure.message.contains("overflow"));
}

#[test]
fn effects_execute_in_source_order() {
    let source = r#"system Test {
  entity Counter { count: Integer }
  action Twice(counter: Counter) {
    effects {
      counter.count += 1
      counter.count += 1
    }
  }
  scenario Ordered {
    given counter.count = 1
    run Twice(counter)
    expect counter.count == 3
  }
}
"#;
    let report = simulate(&checked(source), "Ordered").unwrap();
    assert!(report.succeeded());
    assert_eq!(report.changes.len(), 2);
    assert_eq!(report.state["counter.count"], Value::Integer(3));
}

#[test]
fn run_arguments_bind_positionally_to_distinct_entity_instances() {
    let source = r#"system Test {
  entity Account { balance: Integer }
  action Move(from: Account, to: Account) {
    requires from.balance > 0
    effects {
      from.balance -= 1
      to.balance += 1
    }
  }
  scenario Transfer {
    given source.balance = 2
    given destination.balance = 0
    run Move(source, destination)
    expect source.balance == 1
    expect destination.balance == 1
  }
}
"#;
    let report = simulate(&checked(source), "Transfer").unwrap();
    assert!(report.succeeded());
    assert_eq!(report.state["source.balance"], Value::Integer(1));
    assert_eq!(report.state["destination.balance"], Value::Integer(1));
}

#[test]
fn boolean_state_is_supported() {
    let source = r#"system Test {
  entity Switch { enabled: Boolean }
  action Disable(switch: Switch) { effects switch.enabled = false }
  scenario Toggle {
    given switch.enabled = true
    run Disable(switch)
    expect switch.enabled == false
  }
}
"#;
    let report = simulate(&checked(source), "Toggle").unwrap();
    assert!(report.succeeded());
    assert_eq!(report.state["switch.enabled"], Value::Boolean(false));
}

#[test]
fn unsupported_effect_value_types_fail_static_check_before_simulation() {
    let source = r#"system Test {
  entity Item {
    id: ID
    ready: Boolean
  }
  action Assign(item: Item) { effects item.id = 1 }
  scenario Unsupported {
    given item.ready = true
    run Assign(item)
    expect item.ready == true
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostic = simulate(&document, "Unsupported").expect_err("static check must fail");
    assert_eq!(diagnostic.code, "MORVA2016");
}

#[test]
fn set_effect_type_mismatches_fail_static_check_before_simulation() {
    let source = r#"system Test {
  entity Counter { count: Integer }
  action Corrupt(counter: Counter) {
    effects counter.count = counter.count == 1
  }
  scenario Invalid {
    given counter.count = 1
    run Corrupt(counter)
    expect true
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostic = simulate(&document, "Invalid").expect_err("static check must fail");
    assert_eq!(diagnostic.code, "MORVA2016");
}

#[test]
fn entity_invariants_resolve_contextual_enum_members() {
    let source = r#"system Test {
  enum State { Ready }
  entity Item {
    state: State
    invariant state == Ready
  }
  action Keep(item: Item) {}
  scenario Valid {
    given item.state = Ready
    run Keep(item)
    expect item.state == Ready
  }
}
"#;
    assert!(simulate(&checked(source), "Valid").unwrap().succeeded());
}

#[test]
fn equality_type_mismatches_fail_static_check_before_simulation() {
    let source = r#"system Test {
  entity Item {
    count: Integer
    enabled: Boolean
  }
  action Compare(item: Item) { requires item.count != item.enabled }
  scenario Invalid {
    given item.count = 1
    given item.enabled = true
    run Compare(item)
    expect true
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostic = simulate(&document, "Invalid").expect_err("static check must fail");
    assert_eq!(diagnostic.code, "MORVA2014");
}

#[test]
fn public_simulate_rejects_an_unchecked_invalid_scenario() {
    let source = r#"system Test {
  entity Item { count: Integer }
  action Set(item: Item) { effects item.count = 1 }
  scenario Invalid {
    run Set(item)
    given item.count = 0
    expect true
  }
}
"#;
    let document = parse(source).expect("syntax is valid");
    assert_eq!(
        simulate(&document, "Invalid").unwrap_err().code,
        "MORVA3003"
    );
}

#[test]
fn scenario_structure_is_checked() {
    let missing_run = "system Test { scenario Broken { expect true } }";
    let codes = diagnostic_codes(missing_run);
    assert!(codes.contains(&"MORVA3004"));

    let no_expect = "system Test { action A {} scenario Broken { run A() } }";
    assert!(diagnostic_codes(no_expect).contains(&"MORVA3005"));

    let multiple_run = "system Test {\n  action A {}\n  scenario Broken {\n    run A()\n    run A()\n    expect true\n  }\n}";
    assert!(diagnostic_codes(multiple_run).contains(&"MORVA3004"));

    let wrong_order = "system Test {\n  action A {}\n  scenario Broken {\n    expect true\n    run A()\n    given x.y = 1\n  }\n}";
    let codes = diagnostic_codes(wrong_order);
    assert!(codes.contains(&"MORVA3003"));
    assert!(codes.contains(&"MORVA3005"));
}

#[test]
fn action_selection_and_binding_are_checked() {
    let unknown = "system Test {\n  scenario Broken {\n    run Missing(x)\n    expect true\n  }\n}";
    assert!(diagnostic_codes(unknown).contains(&"MORVA3006"));

    let arity = "system Test {\n  entity Item {}\n  action A(x: Item) {}\n  scenario Broken {\n    run A()\n    expect true\n  }\n}";
    assert!(diagnostic_codes(arity).contains(&"MORVA3007"));

    let duplicate = "system Test {\n  entity Item {}\n  action A(x: Item, y: Item) {}\n  scenario Broken {\n    run A(item, item)\n    expect true\n  }\n}";
    assert!(diagnostic_codes(duplicate).contains(&"MORVA3008"));

    let scalar = "system Test {\n  action A(x: Integer) {}\n  scenario Broken {\n    run A(value)\n    expect true\n  }\n}";
    assert!(diagnostic_codes(scalar).contains(&"MORVA3009"));
}

#[test]
fn action_and_scenario_names_must_be_globally_unique() {
    let source = r#"system Test {
  module A {
    action Do {}
    scenario Case {
      run Do()
      expect true
    }
  }
  module B {
    action Do {}
    scenario Case {
      run Do()
      expect true
    }
  }
}
"#;
    let codes = diagnostic_codes(source);
    assert!(codes.contains(&"MORVA3001"));
    assert!(codes.contains(&"MORVA3002"));
}

#[test]
fn invalid_given_targets_operators_and_values_are_checked() {
    let source = r#"system Test {
  enum State { Ready }
  entity Item {
    state: State
    name: String
  }
  action A(item: Item) {}
  scenario Broken {
    given item.state += 1
    given missing.state = Ready
    given item.name = 1
    given item.state = Missing
    run A(item)
    expect true
  }
}
"#;
    let codes = diagnostic_codes(source);
    assert!(codes.contains(&"MORVA3010"));
    assert!(codes.contains(&"MORVA3011"));
    assert!(codes.contains(&"MORVA3012"));
    assert!(codes.contains(&"MORVA2012"));
}

#[test]
fn unknown_scenario_selection_is_reported() {
    let source =
        "system Test {\n  action A {}\n  scenario Known {\n    run A()\n    expect true\n  }\n}";
    let error = simulate(&checked(source), "Missing").expect_err("selection must fail");
    assert_eq!(error.code, "MORVA4001");
}

#[test]
fn obvious_transition_contradictions_fail_static_check_before_simulation() {
    let source = r#"system Test {
  entity Counter { count: Integer }
  action Impossible(counter: Counter) {
    effects counter.count = 1
    ensures counter.count == 2
  }
  scenario Invalid {
    given counter.count = 0
    run Impossible(counter)
    expect true
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "MORVA2019");
    let diagnostic = simulate(&document, "Invalid").expect_err("static check must fail");
    assert_eq!(diagnostic, diagnostics[0]);
}

#[test]
fn soft_behaviors_do_not_change_any_simulation_result_field() {
    let plain = r#"system Test {
  entity Counter { count: Integer }
  action Increment(counter: Counter) {
    effects counter.count += 1
    ensures counter.count == 2
  }
  scenario Happy {
    given counter.count = 1
    run Increment(counter)
    expect counter.count == 2
  }
}
"#;
    let soft = r#"system Test {
  entity Counter { count: Integer }
  action Increment(counter: Counter) {
    atomic
    idempotent by counter.count
    timeout 30
    retry 2
    implementation_hint {
      adapter { external ignored }
    }
    effects counter.count += 1
    ensures counter.count == 2
  }
  scenario Happy {
    given counter.count = 1
    run Increment(counter)
    expect counter.count == 2
  }
}
"#;

    let plain_report = simulate(&checked(plain), "Happy").unwrap();
    let soft_report = simulate(&checked(soft), "Happy").unwrap();
    assert_eq!(soft_report, plain_report);
    assert_eq!(soft_report.phases.len(), 7);
    assert!(soft_report.succeeded());
}

#[test]
fn simulation_phases_match_the_capability_inventory() {
    let document = checked(include_str!("../../../examples/order.morva"));
    let report = simulate(&document, "NormalConfirmation").expect("scenario selected");
    let inventory = morva_core::capabilities();
    assert_eq!(
        report
            .phases
            .iter()
            .map(|phase| phase.phase.as_str())
            .collect::<Vec<_>>(),
        inventory.simulation_phases
    );
}

#[test]
fn negation_evaluates_deterministically_in_all_phases() {
    let source = r#"system Shop {
  entity Order {
    open: Boolean
  }
  action Close(order: Order) {
    requires !(order.open == false)
    effects order.open = false
    ensures !order.open
  }
  scenario CloseIt {
    given order.open = true
    run Close(order)
    expect !order.open
  }
}
"#;
    let report = simulate(&checked(source), "CloseIt").unwrap();
    assert!(report.succeeded());
    assert_eq!(report.state["order.open"], Value::Boolean(false));

    let failing = r#"system Shop {
  entity Order {
    open: Boolean
  }
  action Close(order: Order) {
    requires !order.open
    effects order.open = false
  }
  scenario CloseIt {
    given order.open = true
    run Close(order)
    expect true
  }
}
"#;
    let report = simulate(&checked(failing), "CloseIt").unwrap();
    let failure = report.failure.as_ref().expect("negated requires fails");
    assert_eq!(failure.phase, SimulationPhase::Requires);
    assert!(report.changes.is_empty());
}

#[test]
fn negation_preserves_the_uninitialized_read_contract() {
    let source = r#"system Shop {
  entity Order {
    open: Boolean
  }
  action Close(order: Order) {
    requires !order.open
    effects order.open = false
  }
  scenario CloseIt {
    run Close(order)
    expect true
  }
}
"#;
    let full_source = source;
    let report = simulate(&checked(source), "CloseIt").unwrap();
    let failure = report.failure.as_ref().expect("uninitialized read fails");
    assert_eq!(failure.phase, SimulationPhase::Requires);
    assert!(failure.message.contains("uninitialized"));
    let path_start = full_source.find("!order.open").unwrap() + 1;
    assert_eq!(
        failure.span.start, path_start,
        "failure span points at the responsible path, not the negation"
    );
}
