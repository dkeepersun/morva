//! Shared machine-output payloads for Morva integrations.
//!
//! The CLI's `--format json` envelope and the MCP tools serve the same
//! versioned payload schema; this crate is the single implementation both
//! consume, so no transport maintains a second diagnostic, AST, summary, or
//! simulation table. It depends only on `morva-core` and stays std-only.

use morva_core::json::Json;
use morva_core::{
    AnalysisFinding, Assignment, ClauseExpression, Declaration, Diagnostic, Document, Expr,
    ExprKind, NoticeKind, Path, Project, ProjectDiagnostic, ProjectFinding, ScenarioItem,
    SimulationPhase, Span, Value, analyze, simulate,
};

pub const MACHINE_PROTOCOL: &str = "morva.cli";
pub const MACHINE_SCHEMA_VERSION: u64 = 1;

/// One caller-visible source: a display name (a file path for the CLI, a
/// logical name for MCP) plus its exact UTF-8 text.
pub struct NamedSource<'a> {
    pub name: &'a str,
    pub text: &'a str,
}

/// A checked model plus the source identities every location maps back to.
pub enum MachineModel<'a> {
    Single {
        source: NamedSource<'a>,
        document: &'a Document,
    },
    Project {
        sources: Vec<NamedSource<'a>>,
        project: &'a Project,
    },
}

impl MachineModel<'_> {
    pub fn document(&self) -> &Document {
        match self {
            Self::Single { document, .. } => document,
            Self::Project { project, .. } => project.document(),
        }
    }

    /// Maps a document span (virtual for projects) to its owning source name,
    /// text, and file-local span.
    fn try_locate(&self, span: Span) -> Option<(&str, &str, Span)> {
        match self {
            Self::Single { source, .. } => Some((source.name, source.text, span)),
            Self::Project { sources, project } => {
                let local = project.locate_virtual_span(span)?;
                let source = &sources[local.source_id.0];
                Some((source.name, source.text, local.local_span))
            }
        }
    }

    fn locate(&self, span: Span) -> Json {
        match self.try_locate(span) {
            Some((name, text, local)) => source_location(name, text, local),
            None => Json::Null,
        }
    }
}

/// 1-based line and column of a byte offset, honoring LF, CRLF, and CR.
pub fn line_and_column(source: &str, offset: usize) -> (usize, usize) {
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
    (line, column)
}

pub fn envelope(command: &str, success: bool, extra: Vec<(&'static str, Json)>) -> Json {
    let mut members = vec![
        ("protocol", Json::string(MACHINE_PROTOCOL)),
        ("schema_version", Json::UInt(MACHINE_SCHEMA_VERSION)),
        ("command", Json::string(command)),
        ("success", Json::Bool(success)),
    ];
    members.extend(extra);
    Json::Object(members)
}

pub fn render(envelope: &Json) -> String {
    let mut out = String::new();
    envelope.write(&mut out, 0);
    out.push('\n');
    out
}

pub fn error_envelope(command: &str, kind: &str, message: &str) -> Json {
    envelope(
        command,
        false,
        vec![(
            "error",
            Json::Object(vec![
                ("kind", Json::string(kind)),
                ("message", Json::string(message)),
            ]),
        )],
    )
}

pub fn source_location(name: &str, text: &str, span: Span) -> Json {
    let (line, column) = line_and_column(text, span.start);
    Json::Object(vec![
        ("source", Json::string(name)),
        ("line", Json::UInt(line as u64)),
        ("column", Json::UInt(column as u64)),
        (
            "span",
            Json::Object(vec![
                ("start", Json::UInt(span.start as u64)),
                ("end", Json::UInt(span.end as u64)),
            ]),
        ),
    ])
}

pub fn diagnostic(
    severity: &str,
    code: &str,
    message: &str,
    location: Option<(&str, &str, Span)>,
) -> Json {
    let location = match location {
        None => Json::Null,
        Some((name, text, span)) => source_location(name, text, span),
    };
    Json::Object(vec![
        ("severity", Json::string(severity)),
        ("code", Json::string(code)),
        ("message", Json::string(message)),
        ("location", location),
    ])
}

pub fn single_parse_diagnostics(source: &NamedSource<'_>, diagnostics: &[Diagnostic]) -> Vec<Json> {
    diagnostics
        .iter()
        .map(|item| {
            diagnostic(
                "error",
                item.code,
                &item.message,
                Some((source.name, source.text, item.span)),
            )
        })
        .collect()
}

pub fn project_diagnostic(sources: &[NamedSource<'_>], item: &ProjectDiagnostic) -> Json {
    match item {
        ProjectDiagnostic::Project { diagnostic: inner } => {
            diagnostic("error", inner.code, &inner.message, None)
        }
        ProjectDiagnostic::Source {
            source_id,
            local_diagnostic,
        } => {
            let source = &sources[source_id.0];
            diagnostic(
                "error",
                local_diagnostic.code,
                &local_diagnostic.message,
                Some((source.name, source.text, local_diagnostic.span)),
            )
        }
    }
}

pub fn project_parse_diagnostics(
    sources: &[NamedSource<'_>],
    diagnostics: &[ProjectDiagnostic],
) -> Vec<Json> {
    diagnostics
        .iter()
        .map(|item| project_diagnostic(sources, item))
        .collect()
}

/// The Story 4.1 check payload: success flag plus the merged findings in the
/// core analysis order.
pub fn check_result(model: &MachineModel<'_>) -> (bool, Vec<Json>) {
    match model {
        MachineModel::Single { source, document } => {
            let report = analyze(document);
            let items = report
                .findings()
                .iter()
                .map(|finding| match finding {
                    AnalysisFinding::Error(error) => diagnostic(
                        "error",
                        error.code,
                        &error.message,
                        Some((source.name, source.text, error.span)),
                    ),
                    AnalysisFinding::Notice(notice) => diagnostic(
                        "warning",
                        notice.code,
                        &notice.message,
                        Some((source.name, source.text, notice.span)),
                    ),
                })
                .collect();
            (!report.has_errors(), items)
        }
        MachineModel::Project { sources, project } => {
            let report = project.analyze();
            let items = report
                .findings()
                .iter()
                .map(|finding| match finding {
                    ProjectFinding::Error(item) => project_diagnostic(sources, item),
                    ProjectFinding::Notice(notice) => {
                        let source = &sources[notice.source_id.0];
                        diagnostic(
                            "warning",
                            notice.local_notice.code,
                            &notice.local_notice.message,
                            Some((source.name, source.text, notice.local_notice.span)),
                        )
                    }
                })
                .collect();
            (!report.has_errors(), items)
        }
    }
}

/// The Story 4.2 structured AST payload. The merged multi-file system shell is
/// synthetic and reports `location: null`.
pub fn ast(model: &MachineModel<'_>) -> Json {
    let document = model.document();
    let system = document
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::System(system) => Some(system),
            _ => None,
        })
        .expect("checked document has one system");
    let system_location = match model {
        MachineModel::Single { .. } => model.locate(system.span),
        MachineModel::Project { .. } => Json::Null,
    };
    Json::Object(vec![
        ("kind", Json::string("system")),
        ("name", Json::string(&system.name.text)),
        ("location", system_location),
        (
            "declarations",
            Json::Array(
                system
                    .declarations
                    .iter()
                    .map(|declaration| ast_declaration(declaration, model))
                    .collect(),
            ),
        ),
    ])
}

fn ast_declaration(declaration: &Declaration, model: &MachineModel<'_>) -> Json {
    match declaration {
        Declaration::System(system) => Json::Object(vec![
            ("kind", Json::string("system")),
            ("name", Json::string(&system.name.text)),
            ("location", model.locate(system.name.span)),
        ]),
        Declaration::Container(container) => Json::Object(vec![
            ("kind", Json::string("container")),
            ("container_kind", Json::string(&container.kind)),
            ("name", Json::string(&container.name.text)),
            ("location", model.locate(container.name.span)),
            (
                "declarations",
                Json::Array(
                    container
                        .declarations
                        .iter()
                        .map(|declaration| ast_declaration(declaration, model))
                        .collect(),
                ),
            ),
        ]),
        Declaration::Enum(enumeration) => Json::Object(vec![
            ("kind", Json::string("enum")),
            ("name", Json::string(&enumeration.name.text)),
            ("location", model.locate(enumeration.span)),
            (
                "members",
                Json::Array(
                    enumeration
                        .members
                        .iter()
                        .map(|member| {
                            Json::Object(vec![
                                ("name", Json::string(&member.text)),
                                ("location", model.locate(member.span)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]),
        Declaration::Entity(entity) => Json::Object(vec![
            ("kind", Json::string("entity")),
            ("name", Json::string(&entity.name.text)),
            ("location", model.locate(entity.span)),
            (
                "fields",
                Json::Array(
                    entity
                        .fields
                        .iter()
                        .map(|field| {
                            Json::Object(vec![
                                ("name", Json::string(&field.name.text)),
                                ("type", Json::string(&field.type_name.text)),
                                ("location", model.locate(field.span)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "invariants",
                Json::Array(
                    entity
                        .invariants
                        .iter()
                        .map(|invariant| ast_expression(invariant, model))
                        .collect(),
                ),
            ),
        ]),
        Declaration::Action(action) => Json::Object(vec![
            ("kind", Json::string("action")),
            ("name", Json::string(&action.name.text)),
            ("location", model.locate(action.span)),
            (
                "parameters",
                Json::Array(
                    action
                        .parameters
                        .iter()
                        .map(|parameter| {
                            Json::Object(vec![
                                ("name", Json::string(&parameter.name.text)),
                                ("type", Json::string(&parameter.type_name.text)),
                                ("location", model.locate(parameter.span)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "soft_behaviors",
                Json::Array(
                    action
                        .soft_behaviors
                        .iter()
                        .map(|behavior| {
                            Json::Object(vec![
                                ("behavior", Json::string(behavior.kind.as_str())),
                                ("location", model.locate(behavior.span)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "clauses",
                Json::Array(
                    action
                        .clauses
                        .iter()
                        .map(|clause| {
                            Json::Object(vec![
                                ("kind", Json::string("clause")),
                                ("clause_kind", Json::string(clause.kind.as_str())),
                                (
                                    "expressions",
                                    Json::Array(
                                        clause
                                            .expressions
                                            .iter()
                                            .map(|expression| match expression {
                                                ClauseExpression::Predicate(predicate) => {
                                                    Json::Object(vec![
                                                        ("kind", Json::string("predicate")),
                                                        (
                                                            "expression",
                                                            ast_expression(predicate, model),
                                                        ),
                                                    ])
                                                }
                                                ClauseExpression::Assignment(assignment) => {
                                                    Json::Object(vec![
                                                        ("kind", Json::string("assignment")),
                                                        (
                                                            "assignment",
                                                            ast_assignment(assignment, model),
                                                        ),
                                                    ])
                                                }
                                            })
                                            .collect(),
                                    ),
                                ),
                                ("location", model.locate(clause.span)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]),
        Declaration::Scenario(scenario) => Json::Object(vec![
            ("kind", Json::string("scenario")),
            ("name", Json::string(&scenario.name.text)),
            ("location", model.locate(scenario.span)),
            (
                "items",
                Json::Array(
                    scenario
                        .items
                        .iter()
                        .map(|item| match item {
                            ScenarioItem::Given(assignment) => Json::Object(vec![
                                ("kind", Json::string("given")),
                                ("assignment", ast_assignment(assignment, model)),
                                ("location", model.locate(assignment.span)),
                            ]),
                            ScenarioItem::Run(run) => Json::Object(vec![
                                ("kind", Json::string("run")),
                                ("action", Json::string(&run.action.text)),
                                (
                                    "arguments",
                                    Json::Array(
                                        run.arguments
                                            .iter()
                                            .map(|argument| {
                                                Json::Object(vec![
                                                    ("name", Json::string(&argument.text)),
                                                    ("location", model.locate(argument.span)),
                                                ])
                                            })
                                            .collect(),
                                    ),
                                ),
                                ("location", model.locate(run.span)),
                            ]),
                            ScenarioItem::Expect(expression) => Json::Object(vec![
                                ("kind", Json::string("expect")),
                                ("expression", ast_expression(expression, model)),
                                ("location", model.locate(expression.span)),
                            ]),
                        })
                        .collect(),
                ),
            ),
        ]),
    }
}

fn ast_assignment(assignment: &Assignment, model: &MachineModel<'_>) -> Json {
    Json::Object(vec![
        ("target", ast_path(&assignment.target, model)),
        ("operator", Json::string(assignment.operator.as_str())),
        ("value", ast_expression(&assignment.value, model)),
        ("location", model.locate(assignment.span)),
    ])
}

fn ast_path(path: &Path, model: &MachineModel<'_>) -> Json {
    Json::Object(vec![
        ("kind", Json::string("path")),
        (
            "segments",
            Json::Array(
                path.segments
                    .iter()
                    .map(|segment| {
                        Json::Object(vec![
                            ("name", Json::string(&segment.text)),
                            ("location", model.locate(segment.span)),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("location", model.locate(path.span)),
    ])
}

fn ast_expression(expression: &Expr, model: &MachineModel<'_>) -> Json {
    match &expression.kind {
        ExprKind::Integer(value) => Json::Object(vec![
            ("kind", Json::string("integer")),
            ("value", Json::string(&value.to_string())),
            ("location", model.locate(expression.span)),
        ]),
        ExprKind::Boolean(value) => Json::Object(vec![
            ("kind", Json::string("boolean")),
            ("value", Json::Bool(*value)),
            ("location", model.locate(expression.span)),
        ]),
        ExprKind::Path(path) => ast_path(path, model),
        ExprKind::Binary {
            left,
            operator,
            right,
        } => Json::Object(vec![
            ("kind", Json::string("binary")),
            ("operator", Json::string(operator.as_str())),
            ("left", ast_expression(left, model)),
            ("right", ast_expression(right, model)),
            ("location", model.locate(expression.span)),
        ]),
        ExprKind::Not(operand) => Json::Object(vec![
            ("kind", Json::string("not")),
            ("operand", ast_expression(operand, model)),
            ("location", model.locate(expression.span)),
        ]),
        ExprKind::Or { left, right } => Json::Object(vec![
            ("kind", Json::string("or")),
            ("left", ast_expression(left, model)),
            ("right", ast_expression(right, model)),
            ("location", model.locate(expression.span)),
        ]),
    }
}

/// The Story 4.3 inspect payload: coverage-warning diagnostics plus the
/// modeled/unmodeled summary.
pub fn inspect(model: &MachineModel<'_>) -> (Vec<Json>, Json) {
    let mut unmodeled_containers = Vec::new();
    let mut unmodeled_behaviors = Vec::new();
    let mut diagnostics = Vec::new();
    let mut push_notice = |kind: &NoticeKind, name: &str, text: &str, span: Span| {
        let entry_location = source_location(name, text, span);
        match kind {
            NoticeKind::CompatibilityContainer { kind, name } => {
                unmodeled_containers.push(Json::Object(vec![
                    ("container_kind", Json::string(kind)),
                    ("name", Json::string(name)),
                    ("location", entry_location),
                ]))
            }
            NoticeKind::ActionSoftBehavior { action, behavior } => {
                unmodeled_behaviors.push(Json::Object(vec![
                    ("action", Json::string(action)),
                    ("behavior", Json::string(behavior.as_str())),
                    ("location", entry_location),
                ]))
            }
        }
    };
    match model {
        MachineModel::Single { source, document } => {
            let report = analyze(document);
            for notice in &report.notices {
                diagnostics.push(diagnostic(
                    "warning",
                    notice.code,
                    &notice.message,
                    Some((source.name, source.text, notice.span)),
                ));
                push_notice(&notice.kind, source.name, source.text, notice.span);
            }
        }
        MachineModel::Project { sources, project } => {
            let report = project.analyze();
            for notice in &report.notices {
                let source = &sources[notice.source_id.0];
                diagnostics.push(diagnostic(
                    "warning",
                    notice.local_notice.code,
                    &notice.local_notice.message,
                    Some((source.name, source.text, notice.local_notice.span)),
                ));
                push_notice(
                    &notice.local_notice.kind,
                    source.name,
                    source.text,
                    notice.local_notice.span,
                );
            }
        }
    }

    let document = model.document();
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

    let modeled = Json::Object(vec![
        (
            "enums",
            Json::Array(
                enumerations
                    .iter()
                    .map(|enumeration| {
                        Json::Object(vec![
                            ("name", Json::string(&enumeration.name.text)),
                            ("member_count", Json::UInt(enumeration.members.len() as u64)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "entities",
            Json::Array(
                entities
                    .iter()
                    .map(|entity| {
                        Json::Object(vec![
                            ("name", Json::string(&entity.name.text)),
                            ("field_count", Json::UInt(entity.fields.len() as u64)),
                            (
                                "invariant_count",
                                Json::UInt(entity.invariants.len() as u64),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "actions",
            Json::Array(
                actions
                    .iter()
                    .map(|action| {
                        let counts = |kind| {
                            action
                                .clauses
                                .iter()
                                .filter(|clause| clause.kind == kind)
                                .map(|clause| clause.expressions.len())
                                .sum::<usize>() as u64
                        };
                        Json::Object(vec![
                            ("name", Json::string(&action.name.text)),
                            (
                                "parameter_count",
                                Json::UInt(action.parameters.len() as u64),
                            ),
                            (
                                "requires",
                                Json::UInt(counts(morva_core::ClauseKind::Requires)),
                            ),
                            (
                                "effects",
                                Json::UInt(counts(morva_core::ClauseKind::Effects)),
                            ),
                            (
                                "ensures",
                                Json::UInt(counts(morva_core::ClauseKind::Ensures)),
                            ),
                            (
                                "invariants",
                                Json::UInt(counts(morva_core::ClauseKind::Invariant)),
                            ),
                            (
                                "soft_behavior_count",
                                Json::UInt(action.soft_behaviors.len() as u64),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "scenarios",
            Json::Array(
                scenarios
                    .iter()
                    .map(|scenario| {
                        let givens = scenario
                            .items
                            .iter()
                            .filter(|item| matches!(item, ScenarioItem::Given(_)))
                            .count() as u64;
                        let expects = scenario
                            .items
                            .iter()
                            .filter(|item| matches!(item, ScenarioItem::Expect(_)))
                            .count() as u64;
                        Json::Object(vec![
                            ("name", Json::string(&scenario.name.text)),
                            ("given_count", Json::UInt(givens)),
                            ("run_count", Json::UInt(1)),
                            ("expect_count", Json::UInt(expects)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ]);
    let container_count = unmodeled_containers.len() as u64;
    let behavior_count = unmodeled_behaviors.len() as u64;
    let summary = Json::Object(vec![
        ("system", Json::string(system)),
        ("modeled", modeled),
        (
            "unmodeled",
            Json::Object(vec![
                ("item_count", Json::UInt(container_count + behavior_count)),
                ("compatibility_container_count", Json::UInt(container_count)),
                (
                    "compatibility_containers",
                    Json::Array(unmodeled_containers),
                ),
                ("action_soft_behavior_count", Json::UInt(behavior_count)),
                ("action_soft_behaviors", Json::Array(unmodeled_behaviors)),
            ]),
        ),
    ]);
    (diagnostics, summary)
}

fn collect_semantic_items<'a>(
    declarations: &'a [Declaration],
    enumerations: &mut Vec<&'a morva_core::Enum>,
    entities: &mut Vec<&'a morva_core::Entity>,
    actions: &mut Vec<&'a morva_core::Action>,
    scenarios: &mut Vec<&'a morva_core::Scenario>,
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

pub enum SimulateOutcome {
    /// The scenario could not be selected; carries one machine diagnostic.
    Selection(Json),
    /// The simulation ran; the report may still describe a failed phase.
    Report { success: bool, report: Json },
}

/// The Story 4.4 simulation payload.
pub fn simulate_report(model: &MachineModel<'_>, scenario: &str) -> SimulateOutcome {
    let report = match simulate(model.document(), scenario) {
        Ok(report) => report,
        Err(item) => {
            return SimulateOutcome::Selection(diagnostic(
                "error",
                item.code,
                &item.message,
                model.try_locate(item.span),
            ));
        }
    };
    let system = model
        .document()
        .declarations
        .iter()
        .find_map(|item| match item {
            Declaration::System(system) => Some(system.name.text.clone()),
            _ => None,
        })
        .expect("checked document has one system");
    let phases = Json::Array(
        SimulationPhase::ALL
            .iter()
            .map(|phase| {
                let status = report
                    .phases
                    .iter()
                    .find(|result| result.phase == *phase)
                    .map_or(
                        "not_run",
                        |result| if result.passed { "passed" } else { "failed" },
                    );
                Json::Object(vec![
                    ("phase", Json::string(phase.as_str())),
                    ("status", Json::string(status)),
                ])
            })
            .collect(),
    );
    let changes = Json::Array(
        report
            .changes
            .iter()
            .map(|change| {
                Json::Object(vec![
                    ("path", Json::string(&change.path)),
                    (
                        "before",
                        change.before.as_ref().map_or(Json::Null, value_json),
                    ),
                    ("after", value_json(&change.after)),
                ])
            })
            .collect(),
    );
    let state = Json::Array(
        report
            .state
            .iter()
            .map(|(path, value)| {
                Json::Object(vec![
                    ("path", Json::string(path)),
                    ("value", value_json(value)),
                ])
            })
            .collect(),
    );
    let failure = match &report.failure {
        None => Json::Null,
        Some(failure) => Json::Object(vec![
            ("phase", Json::string(failure.phase.as_str())),
            ("message", Json::string(&failure.message)),
            ("location", model.locate(failure.span)),
        ]),
    };
    let success = report.succeeded();
    SimulateOutcome::Report {
        success,
        report: Json::Object(vec![
            ("system", Json::string(&system)),
            ("scenario", Json::string(&report.scenario)),
            ("action", Json::string(&report.action)),
            ("phases", phases),
            ("changes", changes),
            ("state", state),
            ("failure", failure),
        ]),
    }
}

fn value_json(value: &Value) -> Json {
    match value {
        Value::Boolean(inner) => Json::Object(vec![
            ("type", Json::string("boolean")),
            ("value", Json::Bool(*inner)),
        ]),
        Value::Integer(inner) => Json::Object(vec![
            ("type", Json::string("integer")),
            ("value", Json::string(&inner.to_string())),
        ]),
        Value::Enum { type_name, member } => Json::Object(vec![
            ("type", Json::string("enum")),
            ("enum", Json::string(type_name)),
            ("member", Json::string(member)),
        ]),
    }
}

/// The Story 4.3 capabilities payload, serialized from the same core
/// inventory as the human command.
pub fn capabilities() -> Json {
    let inventory = morva_core::capabilities();
    let strings =
        |items: &[&'static str]| Json::Array(items.iter().map(|item| Json::string(item)).collect());
    Json::Object(vec![
        ("version", Json::UInt(u64::from(inventory.version))),
        ("declarations", strings(&inventory.declarations)),
        ("clause_kinds", strings(&inventory.clause_kinds)),
        ("expression_forms", strings(&inventory.expression_forms)),
        (
            "comparison_operators",
            strings(&inventory.comparison_operators),
        ),
        (
            "assignment_operators",
            strings(&inventory.assignment_operators),
        ),
        ("literals", strings(&inventory.literals)),
        ("builtin_types", strings(&inventory.builtin_types)),
        (
            "builtin_type_aliases",
            Json::Array(
                inventory
                    .builtin_type_aliases
                    .iter()
                    .map(|(alias, canonical)| {
                        Json::Object(vec![
                            ("alias", Json::string(alias)),
                            ("canonical", Json::string(canonical)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "simulation_value_types",
            strings(&inventory.simulation_value_types),
        ),
        ("simulation_phases", strings(&inventory.simulation_phases)),
        (
            "compatibility_containers",
            strings(&inventory.compatibility_containers),
        ),
        ("soft_behaviors", strings(&inventory.soft_behaviors)),
        ("unsupported", strings(&inventory.unsupported)),
    ])
}
