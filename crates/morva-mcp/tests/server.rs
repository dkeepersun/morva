use std::io::Write;
use std::process::{Command, Stdio};

/// Runs the server binary, feeds it newline-delimited JSON-RPC messages, and
/// returns the raw stdout and stderr after EOF.
fn session(messages: &[&str]) -> (Vec<String>, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_morva-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn morva-mcp");
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        for message in messages {
            stdin.write_all(message.as_bytes()).expect("write message");
            stdin.write_all(b"\n").expect("write newline");
        }
    }
    let output = child.wait_with_output().expect("collect output");
    assert!(output.status.success(), "server exits cleanly on EOF");
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 stdout");
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
    (
        stdout.lines().map(str::to_owned).collect::<Vec<_>>(),
        stderr,
    )
}

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}"#;
const INITIALIZED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

fn tool_call(id: u64, name: &str, sources: &[(&str, &str)], scenario: Option<&str>) -> String {
    let sources = sources
        .iter()
        .map(|(name, text)| {
            format!(
                r#"{{"name":{},"text":{}}}"#,
                json_string(name),
                json_string(text)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let scenario = scenario
        .map(|scenario| format!(r#","scenario":{}"#, json_string(scenario)))
        .unwrap_or_default();
    format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"{name}","arguments":{{"sources":[{sources}]{scenario}}}}}}}"#
    )
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character if (character as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => out.push(character),
        }
    }
    out.push('"');
    out
}

/// Extracts the embedded machine payload text from a tools/call response line.
fn payload_of(line: &str) -> String {
    let marker = "\"text\":\"";
    let start = line.find(marker).expect("tool text content") + marker.len();
    let mut payload = String::new();
    let mut characters = line[start..].chars();
    while let Some(character) = characters.next() {
        match character {
            '"' => break,
            '\\' => match characters.next().expect("escape") {
                'n' => payload.push('\n'),
                't' => payload.push('\t'),
                'r' => payload.push('\r'),
                '"' => payload.push('"'),
                '\\' => payload.push('\\'),
                '/' => payload.push('/'),
                'u' => {
                    let digits: String = (&mut characters).take(4).collect();
                    let unit = u32::from_str_radix(&digits, 16).expect("hex escape");
                    payload.push(char::from_u32(unit).expect("BMP code point"));
                }
                other => panic!("unexpected escape {other:?}"),
            },
            character => payload.push(character),
        }
    }
    payload
}

#[test]
fn initialize_negotiates_and_declares_the_read_only_boundary() {
    let (lines, stderr) = session(&[INITIALIZE, INITIALIZED]);
    assert_eq!(lines.len(), 1, "notifications receive no response");
    assert!(stderr.is_empty(), "no log noise: {stderr}");
    let response = &lines[0];
    assert!(response.contains("\"jsonrpc\":\"2.0\""));
    assert!(response.contains("\"id\":1"));
    assert!(response.contains("\"protocolVersion\":\"2024-11-05\""));
    assert!(response.contains("\"name\":\"morva-mcp\""));
    assert!(response.contains("\"resources\":{}"));
    assert!(response.contains("\"tools\":{}"));
    for promise in [
        "never write",
        "never access the network",
        "reviewed by a human",
    ] {
        assert!(
            response.contains(promise),
            "missing boundary text: {promise}"
        );
    }
}

#[test]
fn unsupported_protocol_versions_get_a_structured_error() {
    let (lines, _) = session(&[
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1999-01-01"}}"#,
    ]);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("\"error\""));
    assert!(lines[0].contains("unsupported protocol version"));
    assert!(lines[0].contains("\"supported\":[\"2024-11-05\""));
}

#[test]
fn protocol_edges_never_panic_or_pollute_stdout() {
    let (lines, stderr) = session(&[
        INITIALIZE,
        "",
        "not json at all",
        r#"{"jsonrpc":"2.0","id":7,"method":"no/such/method"}"#,
        r#"{"jsonrpc":"2.0","id":8,"method":"resources/read","params":{"uri":"morva://missing"}}"#,
        r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"morva_check","arguments":{"sources":[]}}}"#,
    ]);
    assert!(stderr.is_empty());
    assert_eq!(lines.len(), 5);
    for line in &lines {
        assert!(
            line.starts_with('{') && line.ends_with('}'),
            "protocol stdout only: {line}"
        );
    }
    assert!(lines[1].contains("\"-32700\""), "parse error: {}", lines[1]);
    assert!(
        lines[2].contains("\"-32601\""),
        "unknown method: {}",
        lines[2]
    );
    assert!(
        lines[3].contains("\"-32002\""),
        "unknown resource: {}",
        lines[3]
    );
    assert!(
        lines[4].contains("\"-32602\""),
        "invalid params: {}",
        lines[4]
    );
}

#[test]
fn the_capability_resource_serves_the_core_inventory_deterministically() {
    let read = r#"{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"morva://capabilities"}}"#;
    let (first, _) = session(&[INITIALIZE, read]);
    let (second, _) = session(&[INITIALIZE, read]);
    assert_eq!(first[1], second[1], "repeated reads are byte-identical");
    let response = &first[1];
    assert!(response.contains("morva://capabilities"));
    assert!(response.contains("application/json"));
    let text = payload_of_resource(response);
    assert!(text.contains("\"command\": \"capabilities\""));
    assert!(text.contains("\"declarations\": [\n      \"system\","));
    assert!(text.contains("\"soft_behaviors\": [\n      \"atomic\","));
    assert!(text.contains("\"unsupported\": ["));
    assert!(text.contains("\"version\": 1"));
}

fn payload_of_resource(line: &str) -> String {
    payload_of(line)
}

#[test]
fn tools_list_names_the_four_read_only_tools() {
    let (lines, _) = session(&[
        INITIALIZE,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
    ]);
    let response = &lines[1];
    for tool in [
        "morva_check",
        "morva_parse",
        "morva_inspect",
        "morva_simulate",
    ] {
        assert!(
            response.contains(&format!("\"name\":\"{tool}\"")),
            "missing {tool}"
        );
    }
    assert!(response.contains("Read-only"));
    assert!(response.contains("\"required\":[\"sources\"]"));
    assert!(response.contains("\"required\":[\"sources\",\"scenario\"]"));
    assert!(response.contains("never interpreted as a file path"));
}

#[test]
fn morva_check_returns_the_shared_payload_for_memory_projects() {
    let sources: &[(&str, &str)] = &[
        (
            "20-actions.morva",
            "system Shop {\n  action Close(order: Order) {\n    retry 2\n    requires order.open\n  }\n}\n",
        ),
        (
            "10-types.morva",
            "system Shop {\n  entity Order { open: Boolean }\n}\n",
        ),
    ];
    let call = tool_call(2, "morva_check", sources, None);
    let (first, _) = session(&[INITIALIZE, &call]);
    let (second, _) = session(&[INITIALIZE, &call]);
    assert_eq!(first[1], second[1], "byte-identical repeats");
    assert!(first[1].contains("\"isError\":false"));
    let payload = payload_of(&first[1]);
    assert!(payload.contains("\"protocol\": \"morva.cli\""));
    assert!(payload.contains("\"command\": \"check\""));
    assert!(payload.contains("\"success\": true"));
    assert!(payload.contains("\"code\": \"MORVA5002\""));
    assert!(payload.contains("\"source\": \"20-actions.morva\""));
    assert!(!payload.contains("virtual"));

    let broken = tool_call(
        3,
        "morva_check",
        &[(
            "m.morva",
            "system Shop {\n  entity Order { status: Missing }\n}\n",
        )],
        None,
    );
    let (lines, _) = session(&[INITIALIZE, &broken]);
    assert!(
        lines[1].contains("\"isError\":false"),
        "language failure is not a transport failure"
    );
    let payload = payload_of(&lines[1]);
    assert!(payload.contains("\"success\": false"));
    assert!(payload.contains("\"code\": \"MORVA2007\""));
}

#[test]
fn source_bundles_are_validated_before_any_parse() {
    let duplicate = tool_call(
        2,
        "morva_check",
        &[
            ("a.morva", "system Shop {}\n"),
            ("a.morva", "system Shop {}\n"),
        ],
        None,
    );
    let long_name = "n".repeat(1025);
    let oversized_name = tool_call(3, "morva_check", &[(&long_name, "system Shop {}\n")], None);
    let empty_name = tool_call(4, "morva_check", &[("", "system Shop {}\n")], None);
    let (lines, _) = session(&[INITIALIZE, &duplicate, &oversized_name, &empty_name]);
    assert!(lines[1].contains("\"-32602\"") && lines[1].contains("unique"));
    assert!(lines[2].contains("\"-32602\"") && lines[2].contains("1024"));
    assert!(lines[3].contains("\"-32602\"") && lines[3].contains("empty"));

    // 257 sources exceed the bundle limit.
    let names: Vec<String> = (0..257).map(|index| format!("s{index:03}.morva")).collect();
    let many: Vec<(&str, &str)> = names
        .iter()
        .map(|name| (name.as_str(), "system Shop {}\n"))
        .collect();
    let call = tool_call(5, "morva_check", &many, None);
    let (lines, _) = session(&[INITIALIZE, &call]);
    assert!(lines[1].contains("\"-32602\"") && lines[1].contains("256"));
}

#[test]
fn morva_parse_and_inspect_reuse_the_cli_payloads() {
    let sources: &[(&str, &str)] = &[(
        "model.morva",
        "system Shop {\n  module Compat {}\n  entity Order { vip: Boolean\n    paid: Boolean }\n  action Ship(order: Order) {\n    requires !order.paid || order.vip\n  }\n}\n",
    )];
    let parse = tool_call(2, "morva_parse", sources, None);
    let inspect = tool_call(3, "morva_inspect", sources, None);
    let (lines, _) = session(&[INITIALIZE, &parse, &inspect]);
    let ast = payload_of(&lines[1]);
    assert!(ast.contains("\"command\": \"parse\""));
    assert!(ast.contains("\"kind\": \"or\""));
    assert!(ast.contains("\"kind\": \"not\""));
    assert!(ast.contains("\"container_kind\": \"module\""));
    assert!(ast.contains("\"source\": \"model.morva\""));
    let summary = payload_of(&lines[2]);
    assert!(summary.contains("\"command\": \"inspect\""));
    assert!(summary.contains("\"item_count\": 1"));
    assert!(summary.contains("\"container_kind\": \"module\""));
    assert!(summary.contains("\"severity\": \"warning\""));

    // A model error surfaces as a language-level result for both tools.
    let broken: &[(&str, &str)] = &[("bad.morva", "system Shop {\n  action A(x Order) {}\n}\n")];
    let call = tool_call(4, "morva_parse", broken, None);
    let (lines, _) = session(&[INITIALIZE, &call]);
    assert!(lines[1].contains("\"isError\":false"));
    let payload = payload_of(&lines[1]);
    assert!(payload.contains("\"success\": false"));
    assert!(payload.contains("\"severity\": \"error\""));
}

#[test]
fn morva_simulate_reports_phases_and_maps_failures_to_the_responsible_source() {
    let sources: &[(&str, &str)] = &[
        (
            "10-model.morva",
            "system Shop {\n  entity Order { open: Boolean }\n  action Close(order: Order) {\n    requires order.open\n    effects order.open = false\n  }\n}\n",
        ),
        (
            "20-scenarios.morva",
            "system Shop {\n  scenario Closed {\n    given order.open = false\n    run Close(order)\n    expect true\n  }\n}\n",
        ),
    ];
    let failing = tool_call(2, "morva_simulate", sources, Some("Closed"));
    let unknown = tool_call(3, "morva_simulate", sources, Some("Nope"));
    let (lines, _) = session(&[INITIALIZE, &failing, &unknown]);
    let payload = payload_of(&lines[1]);
    assert!(payload.contains("\"command\": \"simulate\""));
    assert!(payload.contains("\"success\": false"));
    assert!(payload.contains("\"phase\": \"requires\",\n        \"status\": \"failed\""));
    assert!(payload.contains("\"status\": \"not_run\""));
    assert!(
        payload.contains("\"source\": \"10-model.morva\""),
        "failure maps to the model source"
    );
    let unknown_payload = payload_of(&lines[2]);
    assert!(unknown_payload.contains("\"code\": \"MORVA4001\""));
    assert!(unknown_payload.contains("\"success\": false"));

    let succeeding: &[(&str, &str)] = &[
        (
            "10-model.morva",
            "system Shop {\n  entity Order { open: Boolean }\n  action Close(order: Order) {\n    requires order.open\n    effects order.open = false\n  }\n}\n",
        ),
        (
            "20-scenarios.morva",
            "system Shop {\n  scenario Happy {\n    given order.open = true\n    run Close(order)\n    expect !order.open\n  }\n}\n",
        ),
    ];
    let call = tool_call(4, "morva_simulate", succeeding, Some("Happy"));
    let (first, _) = session(&[INITIALIZE, &call]);
    let (second, _) = session(&[INITIALIZE, &call]);
    assert_eq!(first[1], second[1], "byte-identical repeats");
    let payload = payload_of(&first[1]);
    assert!(payload.contains("\"success\": true"));
    assert!(payload.contains("\"failure\": null"));
    assert!(payload.contains("\"type\": \"boolean\""));
    let missing_scenario = tool_call(5, "morva_simulate", succeeding, None);
    let (lines, _) = session(&[INITIALIZE, &missing_scenario]);
    assert!(lines[1].contains("\"-32602\""));
}

#[test]
fn requests_are_isolated_and_leave_no_filesystem_trace() {
    // A failing request must not pollute a later clean one in the same session.
    let broken = tool_call(
        2,
        "morva_check",
        &[(
            "bad.morva",
            "system Shop {\n  entity Order { status: Missing }\n}\n",
        )],
        None,
    );
    let clean = tool_call(3, "morva_check", &[("ok.morva", "system Shop {}\n")], None);
    let (lines, stderr) = session(&[INITIALIZE, &broken, &clean]);
    assert!(stderr.is_empty());
    assert!(payload_of(&lines[1]).contains("\"success\": false"));
    let clean_payload = payload_of(&lines[2]);
    assert!(clean_payload.contains("\"success\": true"));
    assert!(clean_payload.contains("\"diagnostics\": []"));
    // The logical source names never touch the filesystem.
    assert!(!std::path::Path::new("bad.morva").exists());
    assert!(!std::path::Path::new("ok.morva").exists());
}
