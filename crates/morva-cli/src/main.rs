use std::env;
use std::fs::{self, File, Metadata};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use morva_core::json::Json;
use morva_core::{
    Action, AnalysisFinding, ClauseExpression, Declaration, Diagnostic, Document, Entity, Enum,
    Expr, ExprKind, Notice, NoticeKind, Project, ProjectDiagnostic, ProjectFinding, Scenario,
    ScenarioItem, SimulationReport, Span, analyze, check, parse, simulate,
};
use morva_machine::{MachineModel, NamedSource};

#[derive(Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Human,
    Json,
}

const MAX_DIAGNOSTIC_WIDTH: usize = 160;
const LEFT_CONTEXT_WIDTH: usize = 72;
const ELLIPSIS: &str = "...";

fn main() -> ExitCode {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let format = match extract_format(&mut args) {
        Ok(format) => format,
        Err(()) => {
            help();
            return ExitCode::from(2);
        }
    };
    match (args.as_slice(), format) {
        ([command], OutputFormat::Human)
            if command == "help" || command == "--help" || command == "-h" =>
        {
            help();
            ExitCode::SUCCESS
        }
        ([command], OutputFormat::Human) if command == "capabilities" => {
            print_capabilities();
            ExitCode::SUCCESS
        }
        ([command, path], OutputFormat::Human)
            if command == "check" || command == "parse" || command == "inspect" =>
        {
            run(command, path)
        }
        ([command, path, scenario], OutputFormat::Human) if command == "simulate" => {
            run_simulation(path, scenario)
        }
        ([command, path], OutputFormat::Json) if command == "check" => run_check_json(path),
        ([command, path], OutputFormat::Json) if command == "parse" => run_parse_json(path),
        ([command, path], OutputFormat::Json) if command == "inspect" => run_inspect_json(path),
        ([command], OutputFormat::Json) if command == "capabilities" => run_capabilities_json(),
        ([command, path, scenario], OutputFormat::Json) if command == "simulate" => {
            run_simulation_json(path, scenario)
        }
        (rest, OutputFormat::Json) => {
            let command = rest.first().map(String::as_str).unwrap_or("");
            machine_error(command, "usage", "unsupported machine command or arguments")
        }
        _ => {
            help();
            ExitCode::from(2)
        }
    }
}

/// Removes a single `--format <value>` pair anywhere after the command.
/// Only `json` is a recognized machine format.
fn extract_format(args: &mut Vec<String>) -> Result<OutputFormat, ()> {
    let Some(position) = args.iter().position(|argument| argument == "--format") else {
        return Ok(OutputFormat::Human);
    };
    if position + 1 >= args.len() || args[position + 1] != "json" {
        return Err(());
    }
    args.drain(position..position + 2);
    if args.iter().any(|argument| argument == "--format") {
        return Err(());
    }
    Ok(OutputFormat::Json)
}

fn emit_machine(envelope: &Json) {
    print!("{}", morva_machine::render(envelope));
}

fn machine_error(command: &str, kind: &str, message: &str) -> ExitCode {
    emit_machine(&morva_machine::error_envelope(command, kind, message));
    ExitCode::from(2)
}

fn machine_diagnostics_exit(command: &str, success: bool, diagnostics: Vec<Json>) -> ExitCode {
    emit_machine(&morva_machine::envelope(
        command,
        success,
        vec![("diagnostics", Json::Array(diagnostics))],
    ));
    if success {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn load_failure_machine(command: &str, failure: LoadFailure) -> ExitCode {
    match failure {
        LoadFailure::Input(message) => machine_error(command, "input", &message),
        LoadFailure::SingleModel {
            path,
            source,
            diagnostics,
        } => {
            let name = path.to_string_lossy().into_owned();
            let items = morva_machine::single_parse_diagnostics(
                &NamedSource {
                    name: &name,
                    text: &source,
                },
                &diagnostics,
            );
            machine_diagnostics_exit(command, false, items)
        }
        LoadFailure::ProjectModel {
            sources,
            diagnostics,
        } => {
            let names = source_names(&sources);
            let named = named_sources(&sources, &names);
            let items = morva_machine::project_parse_diagnostics(&named, &diagnostics);
            machine_diagnostics_exit(command, false, items)
        }
    }
}

fn source_names(sources: &[CliSource]) -> Vec<String> {
    sources
        .iter()
        .map(|source| source.path.to_string_lossy().into_owned())
        .collect()
}

fn named_sources<'a>(sources: &'a [CliSource], names: &'a [String]) -> Vec<NamedSource<'a>> {
    sources
        .iter()
        .zip(names)
        .map(|(source, name)| NamedSource {
            name,
            text: &source.source,
        })
        .collect()
}

fn model_names(model: &LoadedModel) -> Vec<String> {
    match model {
        LoadedModel::Single { source, .. } => vec![source.path.to_string_lossy().into_owned()],
        LoadedModel::Project { sources, .. } => source_names(sources),
    }
}

fn machine_model<'a>(model: &'a LoadedModel, names: &'a [String]) -> MachineModel<'a> {
    match model {
        LoadedModel::Single { source, document } => MachineModel::Single {
            source: NamedSource {
                name: &names[0],
                text: &source.source,
            },
            document,
        },
        LoadedModel::Project { sources, project } => MachineModel::Project {
            sources: named_sources(sources, names),
            project,
        },
    }
}

fn run_check_json(path: &str) -> ExitCode {
    let model = match load_model(path) {
        Ok(model) => model,
        Err(failure) => return load_failure_machine("check", failure),
    };
    let names = model_names(&model);
    let (success, items) = morva_machine::check_result(&machine_model(&model, &names));
    machine_diagnostics_exit("check", success, items)
}

fn run_parse_json(path: &str) -> ExitCode {
    let model = match load_checked_model(path) {
        Ok(model) => model,
        Err(failure) => return load_failure_machine("parse", failure),
    };
    let names = model_names(&model);
    let ast = morva_machine::ast(&machine_model(&model, &names));
    emit_machine(&morva_machine::envelope("parse", true, vec![("ast", ast)]));
    ExitCode::SUCCESS
}

fn run_inspect_json(path: &str) -> ExitCode {
    let model = match load_checked_model(path) {
        Ok(model) => model,
        Err(failure) => return load_failure_machine("inspect", failure),
    };
    let names = model_names(&model);
    let (diagnostics, summary) = morva_machine::inspect(&machine_model(&model, &names));
    emit_machine(&morva_machine::envelope(
        "inspect",
        true,
        vec![
            ("diagnostics", Json::Array(diagnostics)),
            ("summary", summary),
        ],
    ));
    ExitCode::SUCCESS
}

fn run_capabilities_json() -> ExitCode {
    emit_machine(&morva_machine::envelope(
        "capabilities",
        true,
        vec![("capabilities", morva_machine::capabilities())],
    ));
    ExitCode::SUCCESS
}

fn run_simulation_json(path: &str, scenario: &str) -> ExitCode {
    let model = match load_checked_model(path) {
        Ok(model) => model,
        Err(failure) => return load_failure_machine("simulate", failure),
    };
    let names = model_names(&model);
    match morva_machine::simulate_report(&machine_model(&model, &names), scenario) {
        morva_machine::SimulateOutcome::Selection(item) => {
            machine_diagnostics_exit("simulate", false, vec![item])
        }
        morva_machine::SimulateOutcome::Report { success, report } => {
            emit_machine(&morva_machine::envelope(
                "simulate",
                success,
                vec![("diagnostics", Json::Array(Vec::new())), ("report", report)],
            ));
            if success {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}

fn run_simulation(path: &str, scenario: &str) -> ExitCode {
    let model = match load_checked_model(path) {
        Ok(model) => model,
        Err(failure) => return failure.exit_human(),
    };
    let report = match simulate(model.document(), scenario) {
        Ok(report) => report,
        Err(diagnostic) => {
            model.render_virtual_diagnostics(&[diagnostic]);
            return ExitCode::FAILURE;
        }
    };
    print_simulation(&report);
    if let Some(failure) = &report.failure {
        model.render_simulation_failure(failure.phase.as_str(), &failure.message, failure.span);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run(command: &str, path: &str) -> ExitCode {
    if command == "check" {
        return run_check(path);
    }
    let model = match load_checked_model(path) {
        Ok(model) => model,
        Err(failure) => return failure.exit_human(),
    };
    let document = model.document();
    match command {
        "parse" => print_document(document),
        "inspect" => inspect_document(document, &analyze(document).notices),
        _ => println!("ok: {}", safe_path(path)),
    }
    ExitCode::SUCCESS
}

fn run_check(path: &str) -> ExitCode {
    let model = match load_model(path) {
        Ok(model) => model,
        Err(failure) => return failure.exit_human(),
    };
    let has_errors = match &model {
        LoadedModel::Single { source, document } => {
            let report = analyze(document);
            render_analysis(&source.path.to_string_lossy(), &source.source, &report);
            report.has_errors()
        }
        LoadedModel::Project { sources, project } => {
            let report = project.analyze();
            render_project_analysis(sources, &report);
            report.has_errors()
        }
    };
    if has_errors {
        ExitCode::FAILURE
    } else {
        println!("ok: {}", safe_path(path));
        ExitCode::SUCCESS
    }
}

#[derive(Clone)]
struct CliSource {
    path: PathBuf,
    name: String,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileIdentity {
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(not(unix))]
    length: u64,
    #[cfg(not(unix))]
    modified: Option<std::time::SystemTime>,
}

struct DiscoveredSource {
    path: PathBuf,
    name: String,
    identity: FileIdentity,
}

enum LoadedModel {
    Single {
        source: CliSource,
        document: Document,
    },
    Project {
        sources: Vec<CliSource>,
        project: Project,
    },
}

impl LoadedModel {
    fn document(&self) -> &Document {
        match self {
            Self::Single { document, .. } => document,
            Self::Project { project, .. } => project.document(),
        }
    }

    fn render_virtual_diagnostics(&self, diagnostics: &[Diagnostic]) {
        match self {
            Self::Single { source, .. } => {
                render_diagnostics(&source.path.to_string_lossy(), &source.source, diagnostics)
            }
            Self::Project {
                sources, project, ..
            } => {
                for diagnostic in diagnostics {
                    let location = project
                        .locate_virtual_span(diagnostic.span)
                        .expect("core project diagnostic maps to a source");
                    let source = &sources[location.source_id.0];
                    let diagnostic = Diagnostic {
                        span: location.local_span,
                        ..diagnostic.clone()
                    };
                    render_diagnostics(
                        &source.path.to_string_lossy(),
                        &source.source,
                        &[diagnostic],
                    );
                }
            }
        }
    }

    fn render_simulation_failure(&self, phase: &str, message: &str, span: Span) {
        match self {
            Self::Single { source, .. } => render_simulation_failure(
                &source.path.to_string_lossy(),
                &source.source,
                phase,
                message,
                span,
            ),
            Self::Project {
                sources, project, ..
            } => {
                let location = project
                    .locate_virtual_span(span)
                    .expect("core project runtime span maps to a source");
                let source = &sources[location.source_id.0];
                render_simulation_failure(
                    &source.path.to_string_lossy(),
                    &source.source,
                    phase,
                    message,
                    location.local_span,
                );
            }
        }
    }
}

fn load_checked_model(path: &str) -> Result<LoadedModel, LoadFailure> {
    let model = load_model(path)?;
    match &model {
        LoadedModel::Single { source, document } => {
            let diagnostics = check(document);
            if !diagnostics.is_empty() {
                return Err(LoadFailure::SingleModel {
                    path: source.path.clone(),
                    source: source.source.clone(),
                    diagnostics,
                });
            }
        }
        LoadedModel::Project { sources, project } => {
            let diagnostics = project.check();
            if !diagnostics.is_empty() {
                return Err(LoadFailure::ProjectModel {
                    sources: sources.clone(),
                    diagnostics,
                });
            }
        }
    }
    Ok(model)
}

fn load_model(path: &str) -> Result<LoadedModel, LoadFailure> {
    let path = Path::new(path);
    if path.is_dir() {
        load_project(path)
    } else {
        load_single(path)
    }
}

fn load_single(path: &Path) -> Result<LoadedModel, LoadFailure> {
    let source = read_source(path)?;
    let document = match parse(&source.source) {
        Ok(document) => document,
        Err(diagnostics) => {
            return Err(LoadFailure::SingleModel {
                path: source.path,
                source: source.source,
                diagnostics,
            });
        }
    };
    Ok(LoadedModel::Single { source, document })
}

fn load_project(root: &Path) -> Result<LoadedModel, LoadFailure> {
    let canonical_root = fs::canonicalize(root).map_err(|error| {
        input_failure(format_args!(
            "cannot resolve project directory {}: {error}",
            safe_path(&root.to_string_lossy())
        ))
    })?;
    let discovered = discover_project_sources(root)?;
    let mut sources = Vec::with_capacity(discovered.len());
    for source in discovered {
        sources.push(read_project_source(&canonical_root, source)?);
    }
    let project = match Project::parse(
        sources
            .iter()
            .map(|source| (source.name.as_str(), source.source.as_str())),
    ) {
        Ok(project) => project,
        Err(diagnostics) => {
            return Err(LoadFailure::ProjectModel {
                sources,
                diagnostics,
            });
        }
    };
    Ok(LoadedModel::Project { sources, project })
}

fn discover_project_sources(root: &Path) -> Result<Vec<DiscoveredSource>, LoadFailure> {
    let entries = fs::read_dir(root).map_err(|error| {
        input_failure(format_args!(
            "cannot read project directory {}: {error}",
            safe_path(&root.to_string_lossy())
        ))
    })?;
    let mut sources = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            input_failure(format_args!(
                "cannot discover project sources in {}: {error}",
                safe_path(&root.to_string_lossy())
            ))
        })?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            input_failure(format_args!(
                "cannot inspect {}: {error}",
                safe_path(&path.to_string_lossy())
            ))
        })?;
        if !metadata.file_type().is_file() {
            continue;
        }
        let file_name = entry.file_name();
        if !file_name.as_encoded_bytes().ends_with(b".morva") {
            continue;
        }
        let Some(name) = file_name.to_str() else {
            return Err(input_failure(format_args!(
                "project source filename in {} is not valid UTF-8",
                safe_path(&root.to_string_lossy())
            )));
        };
        sources.push(DiscoveredSource {
            path,
            name: name.to_owned(),
            identity: file_identity(&metadata),
        });
    }
    sources.sort_by(|left, right| left.name.as_bytes().cmp(right.name.as_bytes()));
    if sources.is_empty() {
        return Err(input_failure(format_args!(
            "project directory {} contains no .morva source files",
            safe_path(&root.to_string_lossy())
        )));
    }
    Ok(sources)
}

fn read_source(path: &Path) -> Result<CliSource, LoadFailure> {
    let source = fs::read_to_string(path).map_err(|error| {
        input_failure(format_args!(
            "cannot read {}: {error}",
            safe_path(&path.to_string_lossy())
        ))
    })?;
    Ok(CliSource {
        path: path.to_owned(),
        name: path.to_string_lossy().into_owned(),
        source,
    })
}

fn read_project_source(
    canonical_root: &Path,
    discovered: DiscoveredSource,
) -> Result<CliSource, LoadFailure> {
    let before = fs::symlink_metadata(&discovered.path).map_err(|error| {
        input_failure(format_args!(
            "cannot inspect {} before reading: {error}",
            safe_path(&discovered.path.to_string_lossy())
        ))
    })?;
    if !before.file_type().is_file() || file_identity(&before) != discovered.identity {
        return Err(input_failure(format_args!(
            "project source changed during discovery: {}",
            safe_path(&discovered.path.to_string_lossy())
        )));
    }
    let canonical_path = fs::canonicalize(&discovered.path).map_err(|error| {
        input_failure(format_args!(
            "cannot resolve project source {}: {error}",
            safe_path(&discovered.path.to_string_lossy())
        ))
    })?;
    if canonical_path.parent() != Some(canonical_root) {
        return Err(input_failure(format_args!(
            "project source resolves outside its directory: {}",
            safe_path(&discovered.path.to_string_lossy())
        )));
    }

    let mut file = File::open(&discovered.path).map_err(|error| {
        input_failure(format_args!(
            "cannot read {}: {error}",
            safe_path(&discovered.path.to_string_lossy())
        ))
    })?;
    let opened = file.metadata().map_err(|error| {
        input_failure(format_args!(
            "cannot inspect opened project source {}: {error}",
            safe_path(&discovered.path.to_string_lossy())
        ))
    })?;
    let after = fs::symlink_metadata(&discovered.path).map_err(|error| {
        input_failure(format_args!(
            "cannot revalidate project source {}: {error}",
            safe_path(&discovered.path.to_string_lossy())
        ))
    })?;
    if !opened.is_file()
        || !after.file_type().is_file()
        || file_identity(&opened) != discovered.identity
        || file_identity(&after) != discovered.identity
    {
        return Err(input_failure(format_args!(
            "project source changed while opening: {}",
            safe_path(&discovered.path.to_string_lossy())
        )));
    }

    let mut source = String::new();
    file.read_to_string(&mut source).map_err(|error| {
        input_failure(format_args!(
            "cannot read {}: {error}",
            safe_path(&discovered.path.to_string_lossy())
        ))
    })?;
    Ok(CliSource {
        path: discovered.path,
        name: discovered.name,
        source,
    })
}

#[cfg(unix)]
fn file_identity(metadata: &Metadata) -> FileIdentity {
    use std::os::unix::fs::MetadataExt;
    FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    }
}

#[cfg(not(unix))]
fn file_identity(metadata: &Metadata) -> FileIdentity {
    FileIdentity {
        length: metadata.len(),
        modified: metadata.modified().ok(),
    }
}

fn input_failure(message: std::fmt::Arguments<'_>) -> LoadFailure {
    LoadFailure::Input(message.to_string())
}

enum LoadFailure {
    Input(String),
    SingleModel {
        path: PathBuf,
        source: String,
        diagnostics: Vec<Diagnostic>,
    },
    ProjectModel {
        sources: Vec<CliSource>,
        diagnostics: Vec<ProjectDiagnostic>,
    },
}

impl LoadFailure {
    fn exit_human(self) -> ExitCode {
        match self {
            Self::Input(message) => {
                eprintln!("error: {message}");
                ExitCode::from(2)
            }
            Self::SingleModel {
                path,
                source,
                diagnostics,
            } => {
                render_diagnostics(&path.to_string_lossy(), &source, &diagnostics);
                ExitCode::FAILURE
            }
            Self::ProjectModel {
                sources,
                diagnostics,
            } => {
                render_project_diagnostics(&sources, &diagnostics);
                ExitCode::FAILURE
            }
        }
    }
}

fn render_project_diagnostics(sources: &[CliSource], diagnostics: &[ProjectDiagnostic]) {
    for item in diagnostics {
        match item {
            ProjectDiagnostic::Project { diagnostic } => eprintln!("{diagnostic}"),
            ProjectDiagnostic::Source {
                source_id,
                local_diagnostic,
            } => {
                let source = &sources[source_id.0];
                render_diagnostics(
                    &source.path.to_string_lossy(),
                    &source.source,
                    std::slice::from_ref(local_diagnostic),
                );
            }
        }
    }
}

fn render_diagnostics(path: &str, source: &str, diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        render_source_finding(path, source, &diagnostic.to_string(), diagnostic.span);
    }
}

fn render_source_finding(path: &str, source: &str, header: &str, span: Span) {
    let view = source_view(source, span);
    eprintln!("{header}");
    eprintln!("  --> {}:{}:{}", safe_path(path), view.line, view.column);
    eprintln!("   |");
    eprintln!("{:>3} | {}", view.line, view.rendered);
    eprintln!(
        "   | {}{}",
        " ".repeat(view.marker_start),
        "^".repeat(view.marker_len)
    );
}

fn render_analysis(path: &str, source: &str, report: &morva_core::AnalysisReport) {
    for finding in report.findings() {
        match finding {
            AnalysisFinding::Error(error) => {
                render_source_finding(path, source, &error.to_string(), error.span);
            }
            AnalysisFinding::Notice(notice) => render_source_finding(
                path,
                source,
                &format!("warning[{}]: {}", notice.code, notice.message),
                notice.span,
            ),
        }
    }
}

fn render_project_analysis(sources: &[CliSource], report: &morva_core::ProjectAnalysisReport) {
    for finding in report.findings() {
        match finding {
            ProjectFinding::Error(ProjectDiagnostic::Project { diagnostic }) => {
                eprintln!("{diagnostic}");
            }
            ProjectFinding::Error(ProjectDiagnostic::Source {
                source_id,
                local_diagnostic,
            }) => {
                let source = &sources[source_id.0];
                render_source_finding(
                    &source.path.to_string_lossy(),
                    &source.source,
                    &local_diagnostic.to_string(),
                    local_diagnostic.span,
                );
            }
            ProjectFinding::Notice(notice) => {
                let source = &sources[notice.source_id.0];
                render_source_finding(
                    &source.path.to_string_lossy(),
                    &source.source,
                    &format!(
                        "warning[{}]: {}",
                        notice.local_notice.code, notice.local_notice.message
                    ),
                    notice.local_notice.span,
                );
            }
        }
    }
}

fn render_simulation_failure(path: &str, source: &str, phase: &str, message: &str, span: Span) {
    let view = source_view(source, span);
    eprintln!("simulation[{phase}]: {message}");
    eprintln!("  --> {}:{}:{}", safe_path(path), view.line, view.column);
    eprintln!("   |");
    eprintln!("{:>3} | {}", view.line, view.rendered);
    eprintln!(
        "   | {}{}",
        " ".repeat(view.marker_start),
        "^".repeat(view.marker_len)
    );
}

struct SourceView {
    line: usize,
    column: usize,
    rendered: String,
    marker_start: usize,
    marker_len: usize,
}

fn source_view(source: &str, span: Span) -> SourceView {
    let (line, column, line_start) = location(source, span.start);
    let source_line = &source.as_bytes()[line_start..];
    let marker_start = span.start.saturating_sub(line_start);
    let marker_end = span.end.saturating_sub(line_start);
    let (rendered, marker_start, marker_len) =
        render_source_line(source_line, marker_start, marker_end);
    SourceView {
        line,
        column,
        rendered,
        marker_start,
        marker_len,
    }
}

fn safe_path(path: &str) -> String {
    path.chars()
        .flat_map(|character| {
            if character.is_control() {
                character.escape_default().collect::<Vec<_>>()
            } else {
                vec![character]
            }
        })
        .collect()
}

fn location(source: &str, offset: usize) -> (usize, usize, usize) {
    let offset = offset.min(source.len());
    let bytes = source.as_bytes();
    let mut cursor = 0;
    let mut line = 1;
    let mut line_start = 0;
    while cursor < offset {
        match bytes[cursor] {
            b'\r' if bytes.get(cursor + 1) == Some(&b'\n') => {
                if cursor + 1 >= offset {
                    break;
                }
                cursor += 2;
                line += 1;
                line_start = cursor;
            }
            b'\r' | b'\n' => {
                cursor += 1;
                line += 1;
                line_start = cursor;
            }
            _ => cursor += 1,
        }
    }
    let column = source[line_start..offset].chars().count() + 1;
    (line, column, line_start)
}

fn render_source_line(
    line: &[u8],
    marker_start: usize,
    marker_end: usize,
) -> (String, usize, usize) {
    let raw_marker_start = marker_start.min(line.len());
    let marker_at_line_end =
        raw_marker_start == line.len() || matches!(line.get(raw_marker_start), Some(b'\r' | b'\n'));
    let marker_start = if line.get(raw_marker_start) == Some(&b'\n')
        && raw_marker_start > 0
        && line[raw_marker_start - 1] == b'\r'
    {
        raw_marker_start - 1
    } else {
        raw_marker_start
    };
    let marker_end = marker_end
        .max(raw_marker_start.saturating_add(1))
        .min(line.len());

    let mut window_start = marker_start;
    let mut left_width = 0usize;
    while window_start > 0 {
        let width = fragment_width(line[window_start - 1]);
        if left_width.saturating_add(width) > LEFT_CONTEXT_WIDTH {
            break;
        }
        window_start -= 1;
        left_width += width;
    }
    let left_ellipsis = window_start > 0;
    let prefix_width = usize::from(left_ellipsis) * ELLIPSIS.len();
    let excerpt_limit = MAX_DIAGNOSTIC_WIDTH
        .checked_sub(usize::from(marker_at_line_end))
        .expect("diagnostic width leaves room for an EOF marker");
    let available_width = excerpt_limit
        .checked_sub(prefix_width)
        .expect("diagnostic width leaves room for a left ellipsis");

    let mut window_end = window_start;
    let mut content_width = 0usize;
    let mut reached_line_end = false;
    while window_end < line.len() {
        if matches!(line[window_end], b'\r' | b'\n') {
            reached_line_end = true;
            break;
        }
        let width = fragment_width(line[window_end]);
        if content_width.saturating_add(width) > available_width {
            break;
        }
        content_width += width;
        window_end += 1;
    }
    if window_end == line.len() {
        reached_line_end = true;
    }
    let right_ellipsis = !reached_line_end;
    if right_ellipsis {
        let content_limit = available_width
            .checked_sub(ELLIPSIS.len())
            .expect("diagnostic width leaves room for a right ellipsis");
        while content_width > content_limit {
            window_end -= 1;
            content_width -= fragment_width(line[window_end]);
        }
    }

    let mut rendered = String::with_capacity(MAX_DIAGNOSTIC_WIDTH);
    if left_ellipsis {
        rendered.push_str(ELLIPSIS);
    }
    for byte in &line[window_start..window_end] {
        match byte {
            b'\t' => rendered.push_str("    "),
            0x20..=0x7e => rendered.push(*byte as char),
            _ => rendered.push_str(&format!("\\x{byte:02X}")),
        }
    }
    if right_ellipsis {
        rendered.push_str(ELLIPSIS);
    }

    let visual_start = prefix_width
        + line[window_start..marker_start.min(window_end)]
            .iter()
            .map(|byte| fragment_width(*byte))
            .sum::<usize>();
    let visible_marker_end = marker_end.min(window_end);
    let visual_len = if marker_start < visible_marker_end {
        line[marker_start..visible_marker_end]
            .iter()
            .map(|byte| fragment_width(*byte))
            .sum()
    } else {
        1
    };
    (rendered, visual_start, visual_len)
}

fn fragment_width(byte: u8) -> usize {
    match byte {
        b'\t' => 4,
        0x20..=0x7e => 1,
        _ => 4,
    }
}

fn print_document(document: &Document) {
    for declaration in &document.declarations {
        print_declaration(declaration, 0);
    }
}

fn print_declaration(declaration: &Declaration, depth: usize) {
    let indent = "  ".repeat(depth);
    match declaration {
        Declaration::Entity(entity) => print_entity(entity, depth),
        Declaration::Enum(enumeration) => print_enum(enumeration, depth),
        Declaration::Action(action) => print_action(action, depth),
        Declaration::Scenario(scenario) => print_scenario(scenario, depth),
        _ => {
            println!("{indent}{} {}", declaration.kind(), declaration.name().text);
            for child in declaration.declarations() {
                print_declaration(child, depth + 1);
            }
        }
    }
}

fn print_scenario(scenario: &Scenario, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{indent}scenario {}", scenario.name.text);
    for item in &scenario.items {
        match item {
            ScenarioItem::Given(assignment) => println!(
                "{indent}  given {}",
                format_clause_expression(&ClauseExpression::Assignment(assignment.clone()))
            ),
            ScenarioItem::Run(run) => println!(
                "{indent}  run {}({})",
                run.action.text,
                run.arguments
                    .iter()
                    .map(|item| item.text.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ScenarioItem::Expect(expression) => {
                println!("{indent}  expect {}", format_expr(expression));
            }
        }
    }
}

fn print_enum(enumeration: &Enum, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{indent}enum {}", enumeration.name.text);
    for member in &enumeration.members {
        println!("{indent}  member {}", member.text);
    }
}

fn print_entity(entity: &Entity, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{indent}entity {}", entity.name.text);
    for field in &entity.fields {
        println!(
            "{indent}  field {}: {}",
            field.name.text, field.type_name.text
        );
    }
    for invariant in &entity.invariants {
        println!("{indent}  invariant {}", format_expr(invariant));
    }
}

fn print_action(action: &Action, depth: usize) {
    let indent = "  ".repeat(depth);
    let parameters = action
        .parameters
        .iter()
        .map(|item| format!("{}: {}", item.name.text, item.type_name.text))
        .collect::<Vec<_>>()
        .join(", ");
    println!("{indent}action {}({parameters})", action.name.text);
    for clause in &action.clauses {
        for expression in &clause.expressions {
            println!(
                "{indent}  {} {}",
                clause.kind.as_str(),
                format_clause_expression(expression)
            );
        }
    }
}

fn format_clause_expression(expression: &ClauseExpression) -> String {
    match expression {
        ClauseExpression::Predicate(expression) => format_expr(expression),
        ClauseExpression::Assignment(assignment) => {
            format!(
                "{} {} {}",
                assignment.target.display(),
                assignment.operator.as_str(),
                format_expr(&assignment.value)
            )
        }
    }
}

fn format_expr(expression: &Expr) -> String {
    match &expression.kind {
        ExprKind::Integer(value) => value.to_string(),
        ExprKind::Boolean(value) => value.to_string(),
        ExprKind::Path(path) => path.display(),
        ExprKind::Binary {
            left,
            operator,
            right,
        } => {
            format!(
                "{} {} {}",
                format_expr(left),
                operator.as_str(),
                format_expr(right)
            )
        }
        ExprKind::Not(operand) => {
            if matches!(operand.kind, ExprKind::Binary { .. } | ExprKind::Or { .. }) {
                format!("!({})", format_expr(operand))
            } else {
                format!("!{}", format_expr(operand))
            }
        }
        ExprKind::Or { left, right } => {
            format!("{} || {}", format_expr(left), format_expr(right))
        }
    }
}

fn inspect_document(document: &Document, notices: &[Notice]) {
    let mut enumerations = Vec::new();
    let mut entities = Vec::new();
    let mut actions = Vec::new();
    let mut scenarios = Vec::new();
    collect_semantic_items(
        &document.declarations,
        &mut enumerations,
        &mut entities,
        &mut actions,
        &mut scenarios,
    );
    let system = document
        .declarations
        .iter()
        .find_map(|item| match item {
            Declaration::System(system) => Some(system.name.text.as_str()),
            _ => None,
        })
        .expect("checked document has one system");
    println!("system: {system}");
    println!("enums: {}", enumerations.len());
    for enumeration in enumerations {
        println!(
            "  {}: {} member(s)",
            enumeration.name.text,
            enumeration.members.len()
        );
    }
    println!("entities: {}", entities.len());
    for entity in entities {
        println!(
            "  {}: {} field(s), {} invariant(s)",
            entity.name.text,
            entity.fields.len(),
            entity.invariants.len()
        );
    }
    println!("actions: {}", actions.len());
    for action in actions {
        let counts = |kind| {
            action
                .clauses
                .iter()
                .filter(|clause| clause.kind == kind)
                .map(|clause| clause.expressions.len())
                .sum::<usize>()
        };
        println!(
            "  {}: {} parameter(s), {} requires, {} effects, {} ensures, {} invariants",
            action.name.text,
            action.parameters.len(),
            counts(morva_core::ClauseKind::Requires),
            counts(morva_core::ClauseKind::Effects),
            counts(morva_core::ClauseKind::Ensures),
            counts(morva_core::ClauseKind::Invariant)
        );
    }
    println!("scenarios: {}", scenarios.len());
    for scenario in scenarios {
        let givens = scenario
            .items
            .iter()
            .filter(|item| matches!(item, ScenarioItem::Given(_)))
            .count();
        let expects = scenario
            .items
            .iter()
            .filter(|item| matches!(item, ScenarioItem::Expect(_)))
            .count();
        println!(
            "  {}: {} given(s), 1 run, {} expect(s)",
            scenario.name.text, givens, expects
        );
    }
    print_unmodeled_summary(notices);
}

fn print_unmodeled_summary(notices: &[Notice]) {
    if notices.is_empty() {
        return;
    }
    let containers: Vec<_> = notices
        .iter()
        .filter_map(|notice| match &notice.kind {
            NoticeKind::CompatibilityContainer { kind, name } => Some((kind, name)),
            NoticeKind::ActionSoftBehavior { .. } => None,
        })
        .collect();
    let behaviors: Vec<_> = notices
        .iter()
        .filter_map(|notice| match &notice.kind {
            NoticeKind::ActionSoftBehavior { action, behavior } => Some((action, behavior)),
            NoticeKind::CompatibilityContainer { .. } => None,
        })
        .collect();
    println!("unmodeled: {} item(s)", notices.len());
    println!("  compatibility containers: {}", containers.len());
    for (kind, name) in containers {
        println!("    {kind} {name}");
    }
    println!("  action soft behaviors: {}", behaviors.len());
    for (action, behavior) in behaviors {
        println!("    {action}: {}", behavior.as_str());
    }
}

fn collect_semantic_items<'a>(
    declarations: &'a [Declaration],
    enumerations: &mut Vec<&'a Enum>,
    entities: &mut Vec<&'a Entity>,
    actions: &mut Vec<&'a Action>,
    scenarios: &mut Vec<&'a Scenario>,
) {
    for declaration in declarations {
        match declaration {
            Declaration::Enum(enumeration) => enumerations.push(enumeration),
            Declaration::Entity(entity) => entities.push(entity),
            Declaration::Action(action) => actions.push(action),
            Declaration::Scenario(scenario) => scenarios.push(scenario),
            _ => {}
        }
        collect_semantic_items(
            declaration.declarations(),
            enumerations,
            entities,
            actions,
            scenarios,
        );
    }
}

fn print_simulation(report: &SimulationReport) {
    println!("scenario: {}", report.scenario);
    println!("action: {}", report.action);
    println!("phases:");
    for phase in &report.phases {
        println!(
            "  {}: {}",
            phase.phase.as_str(),
            if phase.passed { "PASS" } else { "FAIL" }
        );
    }
    println!("changes:");
    for change in &report.changes {
        let before = change
            .before
            .as_ref()
            .map_or_else(|| "<unset>".to_owned(), ToString::to_string);
        println!("  {}: {before} -> {}", change.path, change.after);
    }
    println!("state:");
    for (path, value) in &report.state {
        println!("  {path}: {value}");
    }
    println!(
        "result: {}",
        if report.succeeded() { "PASS" } else { "FAIL" }
    );
}

fn print_capabilities() {
    let inventory = morva_core::capabilities();
    println!("capabilities: v{}", inventory.version);
    let line = |label: &str, items: &[&str]| println!("{label}: {}", items.join(", "));
    line("declarations", &inventory.declarations);
    line("clauses", &inventory.clause_kinds);
    line("expression forms", &inventory.expression_forms);
    line("comparison operators", &inventory.comparison_operators);
    line("assignment operators", &inventory.assignment_operators);
    line("literals", &inventory.literals);
    line("builtin types", &inventory.builtin_types);
    println!(
        "builtin type aliases: {}",
        inventory
            .builtin_type_aliases
            .iter()
            .map(|(alias, canonical)| format!("{alias} = {canonical}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    line("simulation value types", &inventory.simulation_value_types);
    line("simulation phases", &inventory.simulation_phases);
    line(
        "compatibility containers (parsed, not validated)",
        &inventory.compatibility_containers,
    );
    line(
        "action soft behaviors (parsed, not validated or executed)",
        &inventory.soft_behaviors,
    );
    println!("unsupported:");
    for item in &inventory.unsupported {
        println!("  {item}");
    }
}

fn help() {
    println!(
        "Morva semantic model tools\n\nUsage:\n  morva check [--format json] <file-or-directory>\n  morva parse [--format json] <file-or-directory>\n  morva inspect [--format json] <file-or-directory>\n  morva simulate [--format json] <file-or-directory> <scenario>\n  morva capabilities [--format json]\n  morva help"
    );
}
