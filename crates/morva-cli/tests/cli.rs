use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

fn morva(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_morva"))
        .args(args)
        .output()
        .expect("run morva")
}

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/order.morva")
}

fn temporary_model(label: &str, source: &[u8]) -> PathBuf {
    let unique = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "morva-{label}-{}-{unique}.morva",
        std::process::id()
    ));
    fs::write(&path, source).expect("write fixture");
    path
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8 output")
}

#[test]
fn check_parse_and_inspect_the_example() {
    let example = example();
    let path = example.to_str().expect("UTF-8 path");
    let checked = morva(&["check", path]);
    assert!(checked.status.success(), "{}", text(&checked.stderr));

    let parsed = morva(&["parse", path]);
    assert!(parsed.status.success(), "{}", text(&parsed.stderr));
    let parsed = text(&parsed.stdout);
    assert!(parsed.contains("member Pending"));
    assert!(parsed.contains("field status: OrderStatus"));
    assert!(parsed.contains("action Confirm(order: Order)"));
    assert!(parsed.contains("effects order.status = Confirmed"));
    assert!(parsed.contains("scenario NormalConfirmation"));
    assert!(parsed.contains("run Confirm(order)"));

    let inspected = morva(&["inspect", path]);
    assert!(inspected.status.success(), "{}", text(&inspected.stderr));
    assert_eq!(
        text(&inspected.stdout),
        "system: Shop\nenums: 1\n  OrderStatus: 3 member(s)\nentities: 1\n  Order: 2 field(s), 0 invariant(s)\nactions: 1\n  Confirm: 1 parameter(s), 1 requires, 1 effects, 1 ensures, 0 invariants\nscenarios: 1\n  NormalConfirmation: 1 given(s), 1 run, 1 expect(s)\n"
    );
}

#[test]
fn syntax_and_semantic_errors_exit_one_with_stable_locations() {
    let syntax = temporary_model(
        "syntax",
        b"system Shop {\n  action Confirm(order Order) {}\n}\n",
    );
    let output = morva(&["check", syntax.to_str().expect("UTF-8 path")]);
    fs::remove_file(syntax).expect("remove fixture");
    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(stderr.contains("error[MORVA1008]"));
    assert!(stderr.contains(":2:24"));

    let semantic = temporary_model(
        "semantic",
        b"system Shop {\n  enum State { Open }\n  entity Item { state: State }\n  action Check(item: Item) { requires item.state == Closed }\n}\n",
    );
    let output = morva(&["check", semantic.to_str().expect("UTF-8 path")]);
    fs::remove_file(semantic).expect("remove fixture");
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("error[MORVA2012]"));
}

#[test]
fn tabs_are_expanded_without_changing_the_reported_column() {
    let path = temporary_model(
        "tab",
        b"system Shop {\n\taction Confirm(order Order) {}\n}\n",
    );
    let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
    fs::remove_file(path).expect("remove fixture");
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains(":2:23"));
    assert!(stderr.contains("2 |     action Confirm(order Order) {}"));
    assert!(!stderr.contains('\t'));
}

#[test]
fn control_characters_are_escaped_in_diagnostics() {
    let path = temporary_model("control", b"system Shop {\n  \x01\n}\n");
    let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
    fs::remove_file(path).expect("remove fixture");
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("error[MORVA1001]"));
    assert!(stderr.contains("2 |   \\x01"));
    assert!(!stderr.as_bytes().contains(&1));
}

#[test]
fn usage_and_file_errors_exit_two() {
    assert_eq!(morva(&["unknown"]).status.code(), Some(2));
    assert_eq!(
        morva(&["check", "/definitely/missing/model.morva"])
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn simulate_reports_the_example_transition_and_passes() {
    let example = example();
    let output = morva(&[
        "simulate",
        example.to_str().expect("UTF-8 path"),
        "NormalConfirmation",
    ]);
    assert!(output.status.success(), "{}", text(&output.stderr));
    let stdout = text(&output.stdout);
    assert!(stdout.contains("action: Confirm"));
    assert!(stdout.contains("order.status: Pending -> Confirmed"));
    assert!(stdout.contains("requires: PASS"));
    assert!(stdout.contains("expects: PASS"));
    assert!(stdout.ends_with("result: PASS\n"));
}

#[test]
fn simulation_model_failure_exits_one_and_renders_its_span() {
    let path = temporary_model(
        "simulation-failure",
        br#"system Test {
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
"#,
    );
    let output = morva(&["simulate", path.to_str().expect("UTF-8 path"), "Invalid"]);
    fs::remove_file(path).expect("remove fixture");
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stdout).contains("result: FAIL"));
    let stderr = text(&output.stderr);
    assert!(stderr.contains("simulation[requires]: predicate evaluated to false"));
    assert!(stderr.contains("requires counter.count > 0"));
}

#[test]
fn unknown_simulation_selection_exits_one_but_usage_stays_two() {
    let example = example();
    let output = morva(&["simulate", example.to_str().expect("UTF-8 path"), "Missing"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("error[MORVA4001]"));
    assert_eq!(morva(&["simulate", "only-a-file"]).status.code(), Some(2));
}
