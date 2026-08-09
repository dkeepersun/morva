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

    let inspected = morva(&["inspect", path]);
    assert!(inspected.status.success(), "{}", text(&inspected.stderr));
    assert_eq!(
        text(&inspected.stdout),
        "system: Shop\nenums: 1\n  OrderStatus: 3 member(s)\nentities: 1\n  Order: 2 field(s), 0 invariant(s)\nactions: 1\n  Confirm: 1 parameter(s), 1 requires, 1 effects, 1 ensures, 0 invariants\n"
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
