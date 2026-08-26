//! morva-mcp: a read-only MCP stdio server over the Morva core.
//!
//! Boundary: the tools never read or write files, never run shells or model
//! actions, and never access the network. Every model tool operates only on
//! caller-provided in-memory UTF-8 sources; all model changes stay a separate
//! human-reviewed step outside this server. stdout carries protocol messages
//! only.

mod json_parse;

use std::io::{BufRead, Write};

use json_parse::ParsedJson;
use morva_core::Project;
use morva_core::json::Json;
use morva_machine::{MachineModel, NamedSource, SimulateOutcome};

const SUPPORTED_PROTOCOL_VERSIONS: [&str; 3] = ["2024-11-05", "2025-03-26", "2025-06-18"];
const CAPABILITIES_URI: &str = "morva://capabilities";
const MAX_LINE_BYTES: usize = 32 * 1024 * 1024;
const MAX_SOURCES: usize = 256;
const MAX_SOURCE_NAME_BYTES: usize = 1024;
const MAX_TOTAL_SOURCE_BYTES: usize = 8 * 1024 * 1024;
const READ_ONLY_INSTRUCTIONS: &str = "Morva read-only validation tools. The tools never write \
files, never execute external actions or shells, and never access the network; they operate only \
on the in-memory UTF-8 sources supplied by the caller. Every model change must be reviewed by a \
human and written to disk outside this server.";

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();
    let mut line = String::new();
    loop {
        line.clear();
        match read_bounded_line(&mut input, &mut line) {
            LineOutcome::Eof => return,
            LineOutcome::TooLong => {
                respond(
                    &mut output,
                    error_response(
                        Json::Null,
                        -32600,
                        "message exceeds the transport size limit",
                    ),
                );
                continue;
            }
            LineOutcome::Line => {}
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match json_parse::parse(trimmed) {
            Err(message) => respond(
                &mut output,
                error_response(Json::Null, -32700, &format!("parse error: {message}")),
            ),
            Ok(message) => {
                if let Some(response) = handle_message(&message) {
                    respond(&mut output, response);
                }
            }
        }
    }
}

enum LineOutcome {
    Line,
    Eof,
    TooLong,
}

fn read_bounded_line(input: &mut impl BufRead, line: &mut String) -> LineOutcome {
    let mut buffer = Vec::new();
    let mut total = 0usize;
    loop {
        let chunk = match input.fill_buf() {
            Ok(chunk) => chunk,
            Err(_) => return LineOutcome::Eof,
        };
        if chunk.is_empty() {
            if buffer.is_empty() {
                return LineOutcome::Eof;
            }
            break;
        }
        let newline = chunk.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(chunk.len(), |position| position + 1);
        total += take;
        if total > MAX_LINE_BYTES {
            // Consume through the end of the oversized line, then reject it.
            input.consume(take);
            if newline.is_none() {
                let mut sink = Vec::new();
                let _ = input.read_until(b'\n', &mut sink);
            }
            return LineOutcome::TooLong;
        }
        buffer.extend_from_slice(&chunk[..take]);
        input.consume(take);
        if newline.is_some() {
            break;
        }
    }
    match String::from_utf8(buffer) {
        Ok(text) => {
            line.push_str(&text);
            LineOutcome::Line
        }
        Err(_) => {
            line.push('\u{FFFD}');
            LineOutcome::Line
        }
    }
}

fn respond(output: &mut impl Write, response: Json) {
    let mut text = String::new();
    response.write_compact(&mut text);
    text.push('\n');
    let _ = output.write_all(text.as_bytes());
    let _ = output.flush();
}

fn request_id(message: &ParsedJson) -> Option<Json> {
    match message.get("id") {
        Some(ParsedJson::Str(value)) => Some(Json::Str(value.clone())),
        Some(ParsedJson::Number(raw)) => raw.parse::<u64>().ok().map(Json::UInt),
        _ => None,
    }
}

fn response(id: Json, result: Json) -> Json {
    Json::Object(vec![
        ("jsonrpc", Json::string("2.0")),
        ("id", id),
        ("result", result),
    ])
}

fn error_response(id: Json, code: i64, message: &str) -> Json {
    error_response_with_data(id, code, message, None)
}

fn error_response_with_data(id: Json, code: i64, message: &str, data: Option<Json>) -> Json {
    let code = if code < 0 {
        // The canonical JSON writer only carries unsigned integers; JSON-RPC
        // codes are emitted through a string-free signed path here.
        Json::Str(code.to_string())
    } else {
        Json::UInt(code as u64)
    };
    let mut error = vec![("code", code), ("message", Json::string(message))];
    if let Some(data) = data {
        error.push(("data", data));
    }
    Json::Object(vec![
        ("jsonrpc", Json::string("2.0")),
        ("id", id),
        ("error", Json::Object(error)),
    ])
}

fn handle_message(message: &ParsedJson) -> Option<Json> {
    let method = message.get("method").and_then(ParsedJson::as_str);
    let id = request_id(message);
    let Some(method) = method else {
        return Some(error_response(
            id.unwrap_or(Json::Null),
            -32600,
            "missing method",
        ));
    };
    if method.starts_with("notifications/") {
        return None;
    }
    let Some(id) = id else {
        // Unknown notification without an id: nothing to answer.
        return None;
    };
    let params = message.get("params");
    Some(match method {
        "initialize" => handle_initialize(id, params),
        "ping" => response(id, Json::Object(Vec::new())),
        "resources/list" => handle_resources_list(id),
        "resources/read" => handle_resources_read(id, params),
        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tools_call(id, params),
        _ => error_response(id, -32601, &format!("unknown method '{method}'")),
    })
}

fn handle_initialize(id: Json, params: Option<&ParsedJson>) -> Json {
    let requested = params
        .and_then(|params| params.get("protocolVersion"))
        .and_then(ParsedJson::as_str);
    let Some(requested) = requested else {
        return error_response(id, -32602, "initialize requires params.protocolVersion");
    };
    if !SUPPORTED_PROTOCOL_VERSIONS.contains(&requested) {
        return error_response_with_data(
            id,
            -32602,
            "unsupported protocol version",
            Some(Json::Object(vec![
                (
                    "supported",
                    Json::Array(
                        SUPPORTED_PROTOCOL_VERSIONS
                            .iter()
                            .map(|version| Json::string(version))
                            .collect(),
                    ),
                ),
                ("requested", Json::string(requested)),
            ])),
        );
    }
    response(
        id,
        Json::Object(vec![
            ("protocolVersion", Json::string(requested)),
            (
                "capabilities",
                Json::Object(vec![
                    ("resources", Json::Object(Vec::new())),
                    ("tools", Json::Object(Vec::new())),
                ]),
            ),
            (
                "serverInfo",
                Json::Object(vec![
                    ("name", Json::string("morva-mcp")),
                    ("version", Json::string(env!("CARGO_PKG_VERSION"))),
                ]),
            ),
            ("instructions", Json::string(READ_ONLY_INSTRUCTIONS)),
        ]),
    )
}

fn handle_resources_list(id: Json) -> Json {
    response(
        id,
        Json::Object(vec![(
            "resources",
            Json::Array(vec![Json::Object(vec![
                ("uri", Json::string(CAPABILITIES_URI)),
                ("name", Json::string("Morva capability inventory")),
                (
                    "description",
                    Json::string(
                        "The authoritative, versioned Morva capability inventory: supported \
constructs, compatibility-recognized-only constructs, and explicitly unsupported categories.",
                    ),
                ),
                ("mimeType", Json::string("application/json")),
            ])]),
        )]),
    )
}

fn handle_resources_read(id: Json, params: Option<&ParsedJson>) -> Json {
    let uri = params
        .and_then(|params| params.get("uri"))
        .and_then(ParsedJson::as_str);
    let Some(uri) = uri else {
        return error_response(id, -32602, "resources/read requires params.uri");
    };
    if uri != CAPABILITIES_URI {
        return error_response(id, -32002, &format!("unknown resource '{uri}'"));
    }
    let envelope = morva_machine::envelope(
        "capabilities",
        true,
        vec![("capabilities", morva_machine::capabilities())],
    );
    response(
        id,
        Json::Object(vec![(
            "contents",
            Json::Array(vec![Json::Object(vec![
                ("uri", Json::string(CAPABILITIES_URI)),
                ("mimeType", Json::string("application/json")),
                ("text", Json::Str(morva_machine::render(&envelope))),
            ])]),
        )]),
    )
}

fn source_bundle_schema() -> Json {
    Json::Object(vec![
        ("type", Json::string("array")),
        ("minItems", Json::UInt(1)),
        (
            "items",
            Json::Object(vec![
                ("type", Json::string("object")),
                (
                    "required",
                    Json::Array(vec![Json::string("name"), Json::string("text")]),
                ),
                (
                    "properties",
                    Json::Object(vec![
                        (
                            "name",
                            Json::Object(vec![
                                ("type", Json::string("string")),
                                (
                                    "description",
                                    Json::string(
                                        "Logical source name used only as a diagnostic identity; \
never interpreted as a file path, URL, or command.",
                                    ),
                                ),
                            ]),
                        ),
                        (
                            "text",
                            Json::Object(vec![
                                ("type", Json::string("string")),
                                (
                                    "description",
                                    Json::string("Exact UTF-8 Morva source text."),
                                ),
                            ]),
                        ),
                    ]),
                ),
            ]),
        ),
    ])
}

fn tool_definition(name: &str, description: &str, with_scenario: bool) -> Json {
    let mut properties = vec![("sources", source_bundle_schema())];
    let mut required = vec![Json::string("sources")];
    if with_scenario {
        properties.push((
            "scenario",
            Json::Object(vec![
                ("type", Json::string("string")),
                (
                    "description",
                    Json::string("Name of the scenario to simulate in memory."),
                ),
            ]),
        ));
        required.push(Json::string("scenario"));
    }
    Json::Object(vec![
        ("name", Json::string(name)),
        ("description", Json::string(description)),
        (
            "inputSchema",
            Json::Object(vec![
                ("type", Json::string("object")),
                ("required", Json::Array(required)),
                ("properties", Json::Object(properties)),
            ]),
        ),
    ])
}

fn handle_tools_list(id: Json) -> Json {
    response(
        id,
        Json::Object(vec![(
            "tools",
            Json::Array(vec![
                tool_definition(
                    "morva_check",
                    "Read-only: check an in-memory Morva source bundle and return the structured \
morva.cli check payload (diagnostics with logical source, line/column, and local span). No file, \
shell, or network access.",
                    false,
                ),
                tool_definition(
                    "morva_parse",
                    "Read-only: return the structured AST payload for an in-memory Morva source \
bundle. No file, shell, or network access.",
                    false,
                ),
                tool_definition(
                    "morva_inspect",
                    "Read-only: return the modeled/unmodeled semantic summary payload for an \
in-memory Morva source bundle. No file, shell, or network access.",
                    false,
                ),
                tool_definition(
                    "morva_simulate",
                    "Read-only: simulate one scenario of an in-memory Morva source bundle and \
return the seven-phase report payload. State exists only in request-private memory.",
                    true,
                ),
            ]),
        )]),
    )
}

struct SourceBundle {
    /// (logical name, text) pairs in canonical UTF-8 name-byte order.
    sources: Vec<(String, String)>,
}

fn parse_bundle(params: Option<&ParsedJson>) -> Result<SourceBundle, String> {
    let sources = params
        .and_then(|params| params.get("sources"))
        .and_then(ParsedJson::as_array)
        .ok_or("params.sources must be an array of {name, text} objects")?;
    if sources.is_empty() {
        return Err("params.sources must not be empty".to_owned());
    }
    if sources.len() > MAX_SOURCES {
        return Err(format!("at most {MAX_SOURCES} sources are accepted"));
    }
    let mut bundle = Vec::with_capacity(sources.len());
    let mut total_bytes = 0usize;
    for item in sources {
        let name = item
            .get("name")
            .and_then(ParsedJson::as_str)
            .ok_or("every source requires a string 'name'")?;
        let text = item
            .get("text")
            .and_then(ParsedJson::as_str)
            .ok_or("every source requires a string 'text'")?;
        if name.is_empty() {
            return Err("source names must not be empty".to_owned());
        }
        if name.len() > MAX_SOURCE_NAME_BYTES {
            return Err(format!(
                "source names are limited to {MAX_SOURCE_NAME_BYTES} bytes"
            ));
        }
        total_bytes += text.len();
        if total_bytes > MAX_TOTAL_SOURCE_BYTES {
            return Err(format!(
                "combined source text is limited to {MAX_TOTAL_SOURCE_BYTES} bytes"
            ));
        }
        bundle.push((name.to_owned(), text.to_owned()));
    }
    bundle.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    if bundle.windows(2).any(|window| window[0].0 == window[1].0) {
        return Err("source names must be unique within one request".to_owned());
    }
    Ok(SourceBundle { sources: bundle })
}

fn tool_result(id: Json, envelope: &Json) -> Json {
    response(
        id,
        Json::Object(vec![
            (
                "content",
                Json::Array(vec![Json::Object(vec![
                    ("type", Json::string("text")),
                    ("text", Json::Str(morva_machine::render(envelope))),
                ])]),
            ),
            ("isError", Json::Bool(false)),
        ]),
    )
}

fn diagnostics_envelope(command: &str, success: bool, diagnostics: Vec<Json>) -> Json {
    morva_machine::envelope(
        command,
        success,
        vec![("diagnostics", Json::Array(diagnostics))],
    )
}

fn handle_tools_call(id: Json, params: Option<&ParsedJson>) -> Json {
    let name = params
        .and_then(|params| params.get("name"))
        .and_then(ParsedJson::as_str);
    let Some(name) = name else {
        return error_response(id, -32602, "tools/call requires params.name");
    };
    let command = match name {
        "morva_check" => "check",
        "morva_parse" => "parse",
        "morva_inspect" => "inspect",
        "morva_simulate" => "simulate",
        _ => return error_response(id, -32602, &format!("unknown tool '{name}'")),
    };
    let arguments = params.and_then(|params| params.get("arguments"));
    let bundle = match parse_bundle(arguments) {
        Ok(bundle) => bundle,
        Err(message) => return error_response(id, -32602, &message),
    };
    let scenario = if command == "simulate" {
        let scenario = arguments
            .and_then(|arguments| arguments.get("scenario"))
            .and_then(ParsedJson::as_str);
        let Some(scenario) = scenario else {
            return error_response(id, -32602, "morva_simulate requires a string 'scenario'");
        };
        if scenario.is_empty() || scenario.len() > MAX_SOURCE_NAME_BYTES {
            return error_response(
                id,
                -32602,
                "scenario names must be non-empty and at most 1024 bytes",
            );
        }
        Some(scenario.to_owned())
    } else {
        None
    };

    let named: Vec<NamedSource<'_>> = bundle
        .sources
        .iter()
        .map(|(name, text)| NamedSource { name, text })
        .collect();
    let project = match Project::parse(
        bundle
            .sources
            .iter()
            .map(|(name, text)| (name.as_str(), text.as_str())),
    ) {
        Ok(project) => project,
        Err(diagnostics) => {
            let items = morva_machine::project_parse_diagnostics(&named, &diagnostics);
            return tool_result(id, &diagnostics_envelope(command, false, items));
        }
    };
    let model = MachineModel::Project {
        sources: named,
        project: &project,
    };
    if command == "check" {
        let (success, items) = morva_machine::check_result(&model);
        return tool_result(id, &diagnostics_envelope("check", success, items));
    }
    // parse, inspect, and simulate require a clean semantic check first,
    // mirroring the CLI's load_checked_model gate.
    let diagnostics = project.check();
    if !diagnostics.is_empty() {
        let MachineModel::Project { sources, .. } = &model else {
            unreachable!("bundle models are always projects");
        };
        let items = morva_machine::project_parse_diagnostics(sources, &diagnostics);
        return tool_result(id, &diagnostics_envelope(command, false, items));
    }
    let envelope = match command {
        "parse" => {
            morva_machine::envelope("parse", true, vec![("ast", morva_machine::ast(&model))])
        }
        "inspect" => {
            let (diagnostics, summary) = morva_machine::inspect(&model);
            morva_machine::envelope(
                "inspect",
                true,
                vec![
                    ("diagnostics", Json::Array(diagnostics)),
                    ("summary", summary),
                ],
            )
        }
        "simulate" => {
            let scenario = scenario.expect("simulate always carries a scenario");
            match morva_machine::simulate_report(&model, &scenario) {
                SimulateOutcome::Selection(item) => {
                    diagnostics_envelope("simulate", false, vec![item])
                }
                SimulateOutcome::Report { success, report } => morva_machine::envelope(
                    "simulate",
                    success,
                    vec![("diagnostics", Json::Array(Vec::new())), ("report", report)],
                ),
            }
        }
        _ => unreachable!("command names are closed above"),
    };
    tool_result(id, &envelope)
}
