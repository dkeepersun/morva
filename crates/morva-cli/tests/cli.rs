use std::fs;
use std::ops::Deref;
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

struct TemporaryProject(PathBuf);

impl Deref for TemporaryProject {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temporary_project(label: &str) -> TemporaryProject {
    let unique = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("morva-{label}-{}-{unique}", std::process::id()));
    fs::create_dir(&path).expect("create project fixture");
    TemporaryProject(path)
}

fn write_project_source(project: &Path, name: &str, source: &[u8]) {
    fs::write(project.join(name), source).expect("write project source");
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8 output")
}

fn diagnostic_content(stderr: &str, line_number: usize) -> (&str, &str) {
    let source_prefix = format!("{line_number:>3} | ");
    let mut lines = stderr.lines();
    let source_line = lines
        .by_ref()
        .find(|line| line.starts_with(&source_prefix))
        .expect("source excerpt");
    let marker_line = lines
        .find(|line| line.starts_with("   | ") && line.contains('^'))
        .expect("marker following source excerpt");
    (
        source_line
            .strip_prefix(&source_prefix)
            .expect("source prefix"),
        marker_line.strip_prefix("   | ").expect("marker prefix"),
    )
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
fn long_line_diagnostic_at_start_has_a_bounded_right_window() {
    let mut source = vec![1];
    source.extend(std::iter::repeat_n(b' ', 100_000));
    source.extend_from_slice(b"\nSECOND_LINE_MUST_NOT_BE_SCANNED_INTO_THE_WINDOW\n");
    let path = temporary_model("long-start", &source);
    let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
    fs::remove_file(path).expect("remove fixture");

    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    let (excerpt, marker) = diagnostic_content(&stderr, 1);
    assert!(excerpt.len() <= 160, "excerpt width: {}", excerpt.len());
    assert!(marker.len() <= 160, "marker width: {}", marker.len());
    assert!(excerpt.starts_with("\\x01"));
    assert!(excerpt.ends_with("..."));
    assert!(!excerpt.contains("SECOND_LINE"));
    assert!(marker.starts_with("^^^^"));
}

#[test]
fn diagnostic_window_crops_only_above_the_160_width_threshold() {
    for (width, cropped) in [(159, false), (160, false), (161, true)] {
        let mut source = vec![1];
        source.extend(std::iter::repeat_n(b' ', width - 4));
        let path = temporary_model(&format!("threshold-{width}"), &source);
        let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
        fs::remove_file(path).expect("remove fixture");

        assert_eq!(output.status.code(), Some(1));
        let stderr = text(&output.stderr);
        let (excerpt, marker) = diagnostic_content(&stderr, 1);
        assert_eq!(excerpt.len(), width.min(160));
        assert_eq!(excerpt.ends_with("..."), cropped);
        assert_eq!(marker, "^^^^");
    }
}

#[test]
fn long_line_diagnostic_in_middle_keeps_the_marker_visible() {
    let mut source = b"system Shop {".to_vec();
    source.extend(std::iter::repeat_n(b' ', 50_000));
    source.push(1);
    source.extend(std::iter::repeat_n(b' ', 50_000));
    let path = temporary_model("long-middle", &source);
    let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
    fs::remove_file(path).expect("remove fixture");

    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(stderr.contains(":1:50014"));
    let (excerpt, marker) = diagnostic_content(&stderr, 1);
    assert!(excerpt.len() <= 160, "excerpt width: {}", excerpt.len());
    assert!(marker.len() <= 160, "marker width: {}", marker.len());
    assert!(excerpt.starts_with("..."));
    assert!(excerpt.ends_with("..."));
    assert!(excerpt.contains("\\x01"));
    assert_eq!(marker.find('^'), excerpt.find("\\x01"));
}

#[test]
fn long_line_diagnostic_at_end_has_a_bounded_left_window() {
    let mut source = vec![b' '; 100_000];
    source.push(1);
    let path = temporary_model("long-end", &source);
    let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
    fs::remove_file(path).expect("remove fixture");

    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(stderr.contains(":1:100001"));
    let (excerpt, marker) = diagnostic_content(&stderr, 1);
    assert!(excerpt.len() <= 160, "excerpt width: {}", excerpt.len());
    assert!(marker.len() <= 160, "marker width: {}", marker.len());
    assert!(excerpt.starts_with("..."));
    assert!(excerpt.ends_with("\\x01"));
    assert!(!excerpt.ends_with("..."));
    assert!(marker.ends_with("^^^^"));
}

#[test]
fn diagnostic_window_adds_a_left_ellipsis_only_beyond_72_width() {
    for (left_width, cropped) in [(72, false), (73, true)] {
        let mut source = vec![b' '; left_width];
        source.push(1);
        let path = temporary_model(&format!("left-threshold-{left_width}"), &source);
        let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
        fs::remove_file(path).expect("remove fixture");

        assert_eq!(output.status.code(), Some(1));
        let stderr = text(&output.stderr);
        let (excerpt, marker) = diagnostic_content(&stderr, 1);
        assert_eq!(excerpt.starts_with("..."), cropped);
        assert_eq!(marker.find('^'), excerpt.find("\\x01"));
        assert!(excerpt.ends_with("\\x01"));
    }
}

#[test]
fn long_line_eof_diagnostic_keeps_a_visible_bounded_caret() {
    let mut source = b"system Shop {".to_vec();
    source.extend(std::iter::repeat_n(b' ', 100_000));
    let path = temporary_model("long-eof", &source);
    let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
    fs::remove_file(path).expect("remove fixture");

    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(stderr.contains("error[MORVA1003]"));
    assert!(stderr.contains(":1:100014"));
    let (excerpt, marker) = diagnostic_content(&stderr, 1);
    assert!(excerpt.len() <= 160, "excerpt width: {}", excerpt.len());
    assert!(marker.len() <= 160, "marker width: {}", marker.len());
    assert!(excerpt.starts_with("..."));
    assert_eq!(
        marker.chars().filter(|character| *character == '^').count(),
        1
    );
}

#[test]
fn multiline_span_marks_only_its_bounded_start_line_window() {
    let mut source = b"system One {".to_vec();
    source.extend(std::iter::repeat_n(b' ', 100_000));
    source.extend_from_slice(b"}\nsystem Two {}\n");
    let path = temporary_model("multiline-span", &source);
    let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
    fs::remove_file(path).expect("remove fixture");

    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(stderr.contains("error[MORVA2001]"));
    assert!(stderr.contains(":1:1"));
    let (excerpt, marker) = diagnostic_content(&stderr, 1);
    assert!(excerpt.len() <= 160, "excerpt width: {}", excerpt.len());
    assert!(marker.len() <= 160, "marker width: {}", marker.len());
    assert!(excerpt.starts_with("system One {"));
    assert!(excerpt.ends_with("..."));
    assert!(marker.starts_with('^'));
    assert!(!excerpt.contains("system Two"));
}

#[test]
fn diagnostic_window_preserves_escaped_fragment_boundaries() {
    let mut source = vec![b'\t'; 30];
    source.push(1);
    source.extend(std::iter::repeat_n(b' ', 100));
    let path = temporary_model("fragment-boundary", &source);
    let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
    fs::remove_file(path).expect("remove fixture");

    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(stderr.contains(":1:31"));
    let (excerpt, marker) = diagnostic_content(&stderr, 1);
    assert_eq!(excerpt.len(), 160);
    assert_eq!(marker.len(), 79);
    assert!(excerpt.starts_with("..."));
    assert!(excerpt.ends_with("..."));
    assert!(excerpt.contains("\\x01"));
    assert!(!excerpt.contains('\t'));
    assert_eq!(marker.find('^'), Some(75));
    assert!(marker.ends_with("^^^^"));
}

#[test]
fn long_line_non_ascii_diagnostic_keeps_the_complete_codepoint_escape() {
    let mut source = vec![b' '; 100];
    source.extend_from_slice("é".as_bytes());
    source.extend(std::iter::repeat_n(b' ', 100));
    let path = temporary_model("long-non-ascii", &source);
    let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
    fs::remove_file(path).expect("remove fixture");

    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(stderr.contains("error[MORVA1002]"));
    assert!(stderr.contains(":1:101"));
    let (excerpt, marker) = diagnostic_content(&stderr, 1);
    assert!(excerpt.len() <= 160);
    assert!(marker.len() <= 160);
    assert!(excerpt.contains("\\xC3\\xA9"));
    assert!(!excerpt.contains('é'));
    assert!(marker.ends_with("^^^^^^^^"));
}

#[test]
fn crlf_diagnostic_excludes_the_carriage_return_from_its_excerpt() {
    let path = temporary_model(
        "crlf",
        b"system Shop {\r\n  action Confirm(order Order) {}\r\n}\r\n",
    );
    let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
    fs::remove_file(path).expect("remove fixture");

    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(stderr.contains(":2:24"));
    assert!(stderr.contains("2 |   action Confirm(order Order) {}"));
    assert!(!stderr.contains("\\x0D"));
    assert!(!stderr.contains('\r'));
}

#[test]
fn mixed_newlines_share_one_logical_line_and_excerpt_contract() {
    let path = temporary_model(
        "mixed-newlines",
        b"system Shop {\r  enum State { Open }\r\n  entity Item { state: State }\n  action Check(item: Item) { requires item.state == Missing }\r}\n",
    );
    let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
    fs::remove_file(path).expect("remove fixture");

    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(stderr.contains("error[MORVA2012]"));
    assert!(stderr.contains(":4:53"));
    assert!(stderr.contains("4 |   action Check(item: Item) { requires item.state == Missing }"));
    assert!(!stderr.contains("\\x0D"));
    assert!(!stderr.contains("Missing }\\x0D}"));
}

#[test]
fn eof_after_each_newline_sequence_has_the_same_location_and_caret() {
    for (label, newline) in [("lf", "\n"), ("crlf", "\r\n"), ("cr", "\r")] {
        let source = format!("system Shop {{{newline}");
        let path = temporary_model(&format!("newline-eof-{label}"), source.as_bytes());
        let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
        fs::remove_file(path).expect("remove fixture");

        assert_eq!(output.status.code(), Some(1));
        let stderr = text(&output.stderr);
        assert!(stderr.contains("error[MORVA1003]"));
        assert!(stderr.contains(":2:1"), "{label}: {stderr:?}");
        let (excerpt, marker) = diagnostic_content(&stderr, 2);
        assert_eq!(excerpt, "");
        assert_eq!(marker, "^");
    }
}

#[test]
fn carriage_return_at_eof_is_a_logical_line_terminator() {
    let path = temporary_model(
        "cr-at-eof",
        b"system Shop { action Confirm(order Order) {}\r",
    );
    let output = morva(&["check", path.to_str().expect("UTF-8 path")]);
    fs::remove_file(path).expect("remove fixture");

    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(stderr.contains("error[MORVA1008]"));
    assert!(stderr.contains(":1:36"));
    assert!(stderr.contains("1 | system Shop { action Confirm(order Order) {}"));
    assert!(!stderr.contains("\\x0D"));
    assert!(!stderr.contains('\r'));
}

#[test]
fn long_line_simulation_failure_uses_the_bounded_diagnostic_window() {
    let mut source = b"system Test {\n  entity Counter { count: Integer }\n  action Check(counter: Counter) {\n    requires".to_vec();
    source.extend(std::iter::repeat_n(b' ', 50_000));
    source.extend_from_slice(b"counter.count > 0");
    source.extend(std::iter::repeat_n(b' ', 50_000));
    source.extend_from_slice(
        b"\n  }\n  scenario Invalid {\n    given counter.count = 0\n    run Check(counter)\n    expect true\n  }\n}\n",
    );
    let path = temporary_model("long-simulation", &source);
    let output = morva(&["simulate", path.to_str().expect("UTF-8 path"), "Invalid"]);
    fs::remove_file(path).expect("remove fixture");

    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stdout).contains("result: FAIL"));
    let stderr = text(&output.stderr);
    assert!(stderr.contains("simulation[requires]: predicate evaluated to false"));
    assert!(stderr.contains(":4:50013"));
    let (excerpt, marker) = diagnostic_content(&stderr, 4);
    assert!(excerpt.len() <= 160);
    assert!(marker.len() <= 160);
    assert!(excerpt.starts_with("..."));
    assert!(excerpt.ends_with("..."));
    assert!(excerpt.contains("counter.count > 0"));
    assert_eq!(marker.find('^'), excerpt.find("counter.count > 0"));
}

#[test]
fn control_characters_in_utf8_paths_are_escaped_for_every_cli_result() {
    let valid = temporary_model("path-\n\t\r\u{1b}\u{7f}-success", b"system Shop {}\n");
    let success = morva(&["check", valid.to_str().expect("UTF-8 path")]);
    fs::remove_file(valid).expect("remove fixture");
    assert!(success.status.success());

    let missing = std::env::temp_dir().join(format!(
        "morva-path-\n\t\r\u{1b}\u{7f}-missing-{}-{}-absent.morva",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let read_error = morva(&["check", missing.to_str().expect("UTF-8 path")]);
    assert_eq!(read_error.status.code(), Some(2));
    let simulation_read_error =
        morva(&["simulate", missing.to_str().expect("UTF-8 path"), "Missing"]);
    assert_eq!(simulation_read_error.status.code(), Some(2));

    let invalid = temporary_model("path-\n\t\r\u{1b}\u{7f}-model", b"\x01");
    let model_error = morva(&["check", invalid.to_str().expect("UTF-8 path")]);
    fs::remove_file(invalid).expect("remove fixture");
    assert_eq!(model_error.status.code(), Some(1));

    let runtime = temporary_model(
        "path-\n\t\r\u{1b}\u{7f}-runtime",
        br#"system Test {
  entity Counter { count: Integer }
  action Check(counter: Counter) { requires counter.count > 0 }
  scenario Invalid {
    given counter.count = 0
    run Check(counter)
    expect true
  }
}
"#,
    );
    let runtime_error = morva(&["simulate", runtime.to_str().expect("UTF-8 path"), "Invalid"]);
    fs::remove_file(runtime).expect("remove fixture");
    assert_eq!(runtime_error.status.code(), Some(1));

    for output in [
        text(&success.stdout),
        text(&read_error.stderr),
        text(&simulation_read_error.stderr),
        text(&model_error.stderr),
        text(&runtime_error.stderr),
    ] {
        assert!(output.contains(r"path-\n\t\r\u{1b}\u{7f}-"), "{output:?}");
        assert!(!output.contains("path-\n"), "{output:?}");
        assert!(!output.contains('\t'), "{output:?}");
        assert!(!output.contains('\r'), "{output:?}");
        assert!(!output.contains('\u{1b}'), "{output:?}");
        assert!(!output.contains('\u{7f}'), "{output:?}");
    }
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

#[test]
fn all_commands_accept_a_sorted_multi_file_project() {
    let project = temporary_project("commands");
    write_project_source(
        &project,
        "20-behavior.morva",
        br#"system Shop {
  action Confirm(order: Order) {
    requires order.status == Pending
    effects order.status = Confirmed
    ensures order.status == Confirmed
  }
  scenario Happy {
    given order.status = Pending
    run Confirm(order)
    expect order.status == Confirmed
  }
}
"#,
    );
    write_project_source(
        &project,
        "10-types.morva",
        br#"system Shop {
  enum State {
    Pending
    Confirmed
  }
  entity Order { status: State }
}
"#,
    );
    let path = project.to_str().unwrap();

    let checked = morva(&["check", path]);
    assert!(checked.status.success(), "{}", text(&checked.stderr));
    assert!(text(&checked.stdout).contains("ok:"));

    let parsed = morva(&["parse", path]);
    assert!(parsed.status.success(), "{}", text(&parsed.stderr));
    let parsed = text(&parsed.stdout);
    assert!(parsed.find("enum State").unwrap() < parsed.find("action Confirm").unwrap());

    let inspected = morva(&["inspect", path]);
    assert!(inspected.status.success(), "{}", text(&inspected.stderr));
    assert!(text(&inspected.stdout).contains("actions: 1"));

    let simulated = morva(&["simulate", path, "Happy"]);
    assert!(simulated.status.success(), "{}", text(&simulated.stderr));
    assert!(text(&simulated.stdout).ends_with("result: PASS\n"));
}

#[test]
fn project_discovery_ignores_non_candidates_subdirectories_and_symlinks() {
    let project = temporary_project("filter");
    write_project_source(&project, "model.morva", b"system Shop {}\n");
    write_project_source(&project, "ignored.MORVA", b"system Wrong {}\n");
    write_project_source(&project, "notes.txt", b"system Wrong {}\n");
    let nested = project.join("nested");
    fs::create_dir(&nested).unwrap();
    write_project_source(&nested, "nested.morva", b"system Wrong {}\n");
    #[cfg(unix)]
    std::os::unix::fs::symlink(project.join("model.morva"), project.join("linked.morva")).unwrap();
    #[cfg(unix)]
    {
        let outside = temporary_model("outside-project", b"system Wrong {}\n");
        std::os::unix::fs::symlink(&outside, project.join("outside.morva")).unwrap();
        let output = morva(&["inspect", project.to_str().unwrap()]);
        fs::remove_file(outside).unwrap();
        assert!(output.status.success(), "{}", text(&output.stderr));
    }

    let output = morva(&["inspect", project.to_str().unwrap()]);
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(
        text(&output.stdout),
        "system: Shop\nenums: 0\nentities: 0\nactions: 0\nscenarios: 0\n"
    );
}

#[test]
fn project_errors_use_the_responsible_file_and_local_location() {
    let syntax = temporary_project("project-syntax");
    write_project_source(&syntax, "10-ok.morva", b"system Shop {}\n");
    write_project_source(
        &syntax,
        "20-bad.morva",
        b"system Shop {\n  action Broken(item Item) {}\n}\n",
    );
    let output = morva(&["check", syntax.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = text(&output.stderr);
    assert!(stderr.contains("20-bad.morva:2:22"), "{stderr}");
    assert!(!stderr.contains("10-ok.morva:"));
    let (excerpt, marker) = diagnostic_content(&stderr, 2);
    assert_eq!(excerpt, "  action Broken(item Item) {}");
    assert_eq!(marker, "                     ^^^^");

    let semantic = temporary_project("project-semantic");
    write_project_source(
        &semantic,
        "10-type.morva",
        b"system Shop { entity Item { active: Boolean } }\n",
    );
    write_project_source(
        &semantic,
        "20-action.morva",
        b"system Shop { action Check(item: Item) { requires item.missing } }\n",
    );
    let output = morva(&["check", semantic.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    let missing = "system Shop { action Check(item: Item) { requires item.missing } }"
        .find("missing")
        .unwrap();
    assert!(
        stderr.contains(&format!("20-action.morva:1:{}", missing + 1)),
        "{stderr}"
    );
    let (excerpt, marker) = diagnostic_content(&stderr, 1);
    assert_eq!(
        excerpt,
        "system Shop { action Check(item: Item) { requires item.missing } }"
    );
    assert_eq!(marker, format!("{}^^^^^^^", " ".repeat(missing)));
}

#[test]
fn project_runtime_failure_maps_to_its_local_source() {
    let project = temporary_project("project-runtime");
    write_project_source(
        &project,
        "10-type.morva",
        b"system Shop { entity Counter { count: Integer } }\n",
    );
    write_project_source(
        &project,
        "20-behavior.morva",
        br#"system Shop {
  action Read(counter: Counter) { requires counter.count > 0 }
  scenario Bad {
    given counter.count = 0
    run Read(counter)
    expect true
  }
}
"#,
    );
    let output = morva(&["simulate", project.to_str().unwrap(), "Bad"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stdout).contains("result: FAIL"));
    let stderr = text(&output.stderr);
    let predicate = "  action Read(counter: Counter) { requires counter.count > 0 }"
        .find("counter.count > 0")
        .unwrap();
    assert!(
        stderr.contains(&format!("20-behavior.morva:2:{}", predicate + 1)),
        "{stderr}"
    );
    let (excerpt, marker) = diagnostic_content(&stderr, 2);
    assert_eq!(
        excerpt,
        "  action Read(counter: Counter) { requires counter.count > 0 }"
    );
    assert_eq!(
        marker,
        format!("{}{}", " ".repeat(predicate), "^".repeat(17))
    );
}

#[test]
fn empty_or_non_utf8_projects_exit_two_without_stdout() {
    let empty = temporary_project("empty");
    let output = morva(&["check", empty.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());

    let invalid = temporary_project("invalid-utf8");
    write_project_source(&invalid, "model.morva", b"system Shop {}\n");
    write_project_source(&invalid, "other.morva", b"\xff");
    let output = morva(&["parse", invalid.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(text(&output.stderr).contains("other.morva"));
}

#[cfg(unix)]
#[test]
fn project_diagnostics_escape_control_characters_in_paths() {
    let project = temporary_project("project-path-\n\t\r\u{1b}\u{7f}");
    write_project_source(
        &project,
        "bad-\n\t\r\u{1b}\u{7f}.morva",
        b"system Shop { action Broken(item Item) {} }\n",
    );
    let output = morva(&["check", project.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(stderr.contains(r"project-path-\n\t\r\u{1b}\u{7f}"));
    assert!(stderr.contains(r"bad-\n\t\r\u{1b}\u{7f}.morva"));
    assert!(!stderr.contains("project-path-\n"));
    assert!(!stderr.contains("bad-\n"));
    assert!(!stderr.contains('\t'));
    assert!(!stderr.contains('\r'));
    assert!(!stderr.contains('\u{1b}'));
    assert!(!stderr.contains('\u{7f}'));
}

#[test]
fn project_shell_errors_have_precise_local_diagnostics() {
    for (label, source, message, column, carets) in [
        (
            "missing-system",
            "entity Loose {}\n",
            "must contain one top-level system",
            1,
            "^^^^^^^^^^^^^^^",
        ),
        (
            "multiple-systems",
            "system Shop {}\nsystem Shop {}\n",
            "multiple top-level systems",
            8,
            "^^^^",
        ),
        (
            "mismatched-system",
            "system Other {}\n",
            "does not match expected system 'Shop'",
            8,
            "^^^^^",
        ),
    ] {
        let project = temporary_project(label);
        write_project_source(&project, "10-good.morva", b"system Shop {}\n");
        write_project_source(&project, "20-bad.morva", source.as_bytes());
        let output = morva(&["check", project.to_str().unwrap()]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let stderr = text(&output.stderr);
        assert!(stderr.contains(message), "{label}: {stderr}");
        let expected_line = if label == "multiple-systems" { 2 } else { 1 };
        assert!(
            stderr.contains(&format!("20-bad.morva:{expected_line}:{column}")),
            "{label}: {stderr}"
        );
        let (excerpt, marker) = diagnostic_content(&stderr, expected_line);
        assert_eq!(excerpt, source.lines().nth(expected_line - 1).unwrap());
        assert_eq!(marker, format!("{}{}", " ".repeat(column - 1), carets));
    }
}

#[test]
fn project_filenames_use_utf8_byte_order() {
    let project = temporary_project("utf8-order");
    write_project_source(
        &project,
        "z.morva",
        b"system Shop {\n  enum Zed {\n    Item\n  }\n}\n",
    );
    write_project_source(
        &project,
        "é.morva",
        b"system Shop {\n  enum Accent {\n    Item\n  }\n}\n",
    );
    let output = morva(&["parse", project.to_str().unwrap()]);
    assert!(output.status.success(), "{}", text(&output.stderr));
    let stdout = text(&output.stdout);
    assert!(stdout.find("enum Zed").unwrap() < stdout.find("enum Accent").unwrap());
}

#[cfg(target_os = "linux")]
#[test]
fn non_utf8_candidate_filename_exits_two_without_stdout() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let project = temporary_project("non-utf8-name");
    let name = OsString::from_vec(b"bad-\xff.morva".to_vec());
    fs::write(project.join(name), b"system Shop {}\n").unwrap();
    let output = morva(&["check", project.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = text(&output.stderr);
    assert!(stderr.contains("filename"));
    assert!(stderr.contains("not valid UTF-8"));
}

#[cfg(unix)]
#[test]
fn unreadable_project_source_exits_two_without_stdout() {
    use std::os::unix::fs::PermissionsExt;

    let project = temporary_project("unreadable");
    let source = project.join("model.morva");
    fs::write(&source, b"system Shop {}\n").unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o000)).unwrap();
    let output = morva(&["check", project.to_str().unwrap()]);
    fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(text(&output.stderr).contains("cannot read"));
}

#[cfg(unix)]
#[test]
fn unreadable_project_directory_exits_two_without_stdout() {
    use std::os::unix::fs::PermissionsExt;

    let project = temporary_project("unreadable-directory");
    fs::set_permissions(&*project, fs::Permissions::from_mode(0o000)).unwrap();
    let output = morva(&["check", project.to_str().unwrap()]);
    fs::set_permissions(&*project, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(text(&output.stderr).contains("cannot read project directory"));
}
