//! Checked-semantics protocol v1: the read-only machine boundary that exposes
//! checker-owned facts (single-file production slice).
//!
//! The projection derives every semantic fact from the existing parser,
//! checker, and analyzer; it never re-implements language rules. Documents are
//! validated against the v1 envelope invariants before serialization, and
//! canonical JSON emission is deterministic byte-for-byte.

use std::collections::HashMap;

use crate::analysis::{self, NoticeKind};
use crate::ast::*;
use crate::semantic::{self, ResolvedType, TypeIndex};
use crate::sha256;

pub const PROTOCOL_NAME: &str = "morva.checked-semantics";
pub const PROTOCOL_VERSION: u64 = 1;
pub const PROTOCOL_CAPABILITIES: [&str; 5] = [
    "morva.sources.inline",
    "morva.locations.byte-range",
    "morva.findings.v1",
    "morva.coverage.v1",
    "morva.checked-model.v1",
];
pub const PROTOCOL_LANGUAGE: &str = "morva";
pub const PROTOCOL_LANGUAGE_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedSemanticsDocument {
    pub producer_version: String,
    pub subject: Subject,
    pub sources: Vec<SourceRecord>,
    pub result: ProtocolResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectKind {
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subject {
    pub kind: SubjectKind,
    pub name: String,
    pub revision: Digest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRecord {
    pub source_id: String,
    pub name: String,
    pub content: String,
    pub revision: Digest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolLocation {
    Source {
        source_id: String,
        byte_range: ByteRange,
    },
    Subject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingCategory {
    Lexical,
    Syntax,
    Project,
    Semantic,
    Coverage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverageDetails {
    CompatibilityContainer {
        container_kind: String,
        name: String,
    },
    ActionSoftBehavior {
        action: String,
        behavior: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub finding_id: String,
    pub severity: Severity,
    pub category: FindingCategory,
    pub code: String,
    pub message: String,
    pub primary_location: ProtocolLocation,
    pub details: Option<CoverageDetails>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageAssessment {
    Complete,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnmodeledItem {
    pub details: CoverageDetails,
    pub location: ProtocolLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coverage {
    pub assessment: CoverageAssessment,
    pub fully_modeled: Option<bool>,
    pub unmodeled: Vec<UnmodeledItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultStatus {
    Valid,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolResult {
    pub status: ResultStatus,
    pub findings: Vec<Finding>,
    pub coverage: Coverage,
    pub checked_model: Option<CheckedModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRef {
    Builtin { name: &'static str },
    Enum { semantic_key: String },
    Entity { semantic_key: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerRef {
    pub container_kind: String,
    pub name: String,
    pub location: ProtocolLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedModel {
    pub system: SystemNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemNode {
    pub node_id: String,
    pub semantic_key: String,
    pub name: String,
    pub shell_locations: Vec<ProtocolLocation>,
    pub declarations: Vec<DeclarationNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclarationNode {
    Enum(EnumNode),
    Entity(EntityNode),
    Action(ActionNode),
    Scenario(ScenarioNode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumNode {
    pub node_id: String,
    pub semantic_key: String,
    pub name: String,
    pub location: ProtocolLocation,
    pub container_path: Vec<ContainerRef>,
    pub members: Vec<EnumMemberNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumMemberNode {
    pub node_id: String,
    pub semantic_key: String,
    pub name: String,
    pub location: ProtocolLocation,
    pub enum_semantic_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityNode {
    pub node_id: String,
    pub semantic_key: String,
    pub name: String,
    pub location: ProtocolLocation,
    pub container_path: Vec<ContainerRef>,
    pub fields: Vec<FieldNode>,
    pub invariants: Vec<ExpressionNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldNode {
    pub node_id: String,
    pub semantic_key: String,
    pub name: String,
    pub location: ProtocolLocation,
    pub field_type: TypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionNode {
    pub node_id: String,
    pub semantic_key: String,
    pub name: String,
    pub location: ProtocolLocation,
    pub container_path: Vec<ContainerRef>,
    pub parameters: Vec<ParameterNode>,
    pub clauses: Vec<ClauseNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterNode {
    pub node_id: String,
    pub semantic_key: String,
    pub name: String,
    pub location: ProtocolLocation,
    pub action_semantic_key: String,
    pub parameter_type: TypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClauseNode {
    pub clause_kind: &'static str,
    pub state_phase: &'static str,
    pub expressions: Vec<ClauseExpressionNode>,
    pub location: ProtocolLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClauseExpressionNode {
    Predicate(ExpressionNode),
    Assignment(Box<AssignmentNode>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentNode {
    pub target: PathExpressionNode,
    pub operator: &'static str,
    pub value: ExpressionNode,
    pub target_type: TypeRef,
    pub location: ProtocolLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathRootNode {
    ActionParameter {
        semantic_key: String,
    },
    EntitySelf {
        semantic_key: String,
    },
    ScenarioInstance {
        name: String,
        instance_type: TypeRef,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathSegmentNode {
    pub name: String,
    pub resolved_type: TypeRef,
    pub location: ProtocolLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathExpressionNode {
    pub root: PathRootNode,
    pub segments: Vec<PathSegmentNode>,
    pub resolved_type: TypeRef,
    pub location: ProtocolLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionNode {
    Integer {
        value: String,
        resolved_type: TypeRef,
        location: ProtocolLocation,
    },
    Boolean {
        value: bool,
        resolved_type: TypeRef,
        location: ProtocolLocation,
    },
    EnumMember {
        enum_semantic_key: String,
        member_semantic_key: String,
        member: String,
        resolved_type: TypeRef,
        location: ProtocolLocation,
    },
    Path(PathExpressionNode),
    Binary {
        operator: &'static str,
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
        resolved_type: TypeRef,
        location: ProtocolLocation,
    },
    Not {
        operand: Box<ExpressionNode>,
        resolved_type: TypeRef,
        location: ProtocolLocation,
    },
    Or {
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
        resolved_type: TypeRef,
        location: ProtocolLocation,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunArgumentNode {
    pub name: String,
    pub parameter_semantic_key: String,
    pub argument_type: TypeRef,
    pub location: ProtocolLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioItemNode {
    Given {
        assignment: Box<AssignmentNode>,
        location: ProtocolLocation,
    },
    Run {
        action_semantic_key: String,
        arguments: Vec<RunArgumentNode>,
        location: ProtocolLocation,
    },
    Expect {
        expression: ExpressionNode,
        location: ProtocolLocation,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioNode {
    pub node_id: String,
    pub semantic_key: String,
    pub name: String,
    pub location: ProtocolLocation,
    pub container_path: Vec<ContainerRef>,
    pub items: Vec<ScenarioItemNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolBuildError {
    InvalidLogicalName(String),
    Projection(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolInvariantError {
    Envelope(String),
    Source(String),
    Location(String),
    Status(String),
    Coverage(String),
    Identity(String),
    Reference(String),
    Model(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolSerializationError(pub ProtocolInvariantError);

pub fn checked_semantics_single_file(
    logical_name: &str,
    source: &str,
) -> Result<CheckedSemanticsDocument, ProtocolBuildError> {
    if logical_name.is_empty() || logical_name.contains('/') || logical_name.contains('\\') {
        return Err(ProtocolBuildError::InvalidLogicalName(format!(
            "logical source name must be a non-empty final path component without separators, got {logical_name:?}"
        )));
    }
    let revision = Digest {
        value: sha256::hex_digest(source.as_bytes()),
    };
    let subject = Subject {
        kind: SubjectKind::File,
        name: logical_name.to_owned(),
        revision: revision.clone(),
    };
    let sources = vec![SourceRecord {
        source_id: "source:0".to_owned(),
        name: logical_name.to_owned(),
        content: source.to_owned(),
        revision,
    }];

    let result = match crate::lexer::lex(source) {
        Err(diagnostics) => failed_result(&diagnostics, FindingCategory::Lexical),
        Ok(_) => match crate::parser::parse(source) {
            Err(diagnostics) => failed_result(&diagnostics, FindingCategory::Syntax),
            Ok(document) => analyzed_result(&document)?,
        },
    };

    Ok(CheckedSemanticsDocument {
        producer_version: env!("CARGO_PKG_VERSION").to_owned(),
        subject,
        sources,
        result,
    })
}

fn source_location(span: Span) -> ProtocolLocation {
    ProtocolLocation::Source {
        source_id: "source:0".to_owned(),
        byte_range: ByteRange {
            start: span.start,
            end: span.end,
        },
    }
}

fn failed_result(diagnostics: &[crate::Diagnostic], category: FindingCategory) -> ProtocolResult {
    let findings = diagnostics
        .iter()
        .enumerate()
        .map(|(index, diagnostic)| Finding {
            finding_id: format!("finding:{index}"),
            severity: Severity::Error,
            category,
            code: diagnostic.code.to_owned(),
            message: diagnostic.message.clone(),
            primary_location: source_location(diagnostic.span),
            details: None,
        })
        .collect();
    ProtocolResult {
        status: ResultStatus::Invalid,
        findings,
        coverage: Coverage {
            assessment: CoverageAssessment::Unavailable,
            fully_modeled: None,
            unmodeled: Vec::new(),
        },
        checked_model: None,
    }
}

fn coverage_details(kind: &NoticeKind) -> CoverageDetails {
    match kind {
        NoticeKind::CompatibilityContainer { kind, name } => {
            CoverageDetails::CompatibilityContainer {
                container_kind: kind.clone(),
                name: name.clone(),
            }
        }
        NoticeKind::ActionSoftBehavior { action, behavior } => {
            CoverageDetails::ActionSoftBehavior {
                action: action.clone(),
                behavior: behavior.as_str(),
            }
        }
    }
}

fn analyzed_result(document: &Document) -> Result<ProtocolResult, ProtocolBuildError> {
    let report = analysis::analyze(document);
    let mut findings = Vec::new();
    for (index, finding) in report.findings().iter().enumerate() {
        findings.push(match finding {
            analysis::AnalysisFinding::Error(error) => Finding {
                finding_id: format!("finding:{index}"),
                severity: Severity::Error,
                category: FindingCategory::Semantic,
                code: error.code.to_owned(),
                message: error.message.clone(),
                primary_location: source_location(error.span),
                details: None,
            },
            analysis::AnalysisFinding::Notice(notice) => Finding {
                finding_id: format!("finding:{index}"),
                severity: Severity::Warning,
                category: FindingCategory::Coverage,
                code: notice.code.to_owned(),
                message: notice.message.clone(),
                primary_location: source_location(notice.span),
                details: Some(coverage_details(&notice.kind)),
            },
        });
    }
    let unmodeled = report
        .notices
        .iter()
        .map(|notice| UnmodeledItem {
            details: coverage_details(&notice.kind),
            location: source_location(notice.span),
        })
        .collect::<Vec<_>>();
    let coverage = Coverage {
        assessment: CoverageAssessment::Complete,
        fully_modeled: Some(unmodeled.is_empty()),
        unmodeled,
    };
    if report.has_errors() {
        return Ok(ProtocolResult {
            status: ResultStatus::Invalid,
            findings,
            coverage,
            checked_model: None,
        });
    }
    let model = project_model(document)?;
    Ok(ProtocolResult {
        status: ResultStatus::Valid,
        findings,
        coverage,
        checked_model: Some(model),
    })
}

struct Projector<'a> {
    index: TypeIndex<'a>,
    actions: HashMap<&'a str, &'a Action>,
    next_node: usize,
}

fn projection_error(message: impl Into<String>) -> ProtocolBuildError {
    ProtocolBuildError::Projection(message.into())
}

impl<'a> Projector<'a> {
    fn node_id(&mut self) -> String {
        let id = format!("node:{}", self.next_node);
        self.next_node += 1;
        id
    }

    fn type_ref(&self, resolved: ResolvedType<'a>) -> TypeRef {
        match resolved {
            ResolvedType::Builtin(builtin) => TypeRef::Builtin {
                name: match builtin.display() {
                    "ID" => "Id",
                    canonical => canonical,
                },
            },
            ResolvedType::Enum(enumeration) => TypeRef::Enum {
                semantic_key: format!("enum:{}", enumeration.name.text),
            },
            ResolvedType::Entity(entity) => TypeRef::Entity {
                semantic_key: format!("entity:{}", entity.name.text),
            },
        }
    }

    fn named_type(&self, name: &Name) -> Result<ResolvedType<'a>, ProtocolBuildError> {
        semantic::resolve_type(name, &self.index)
            .ok_or_else(|| projection_error(format!("unresolved type '{}'", name.text)))
    }
}

#[derive(Clone, Copy)]
enum ExpressionScope<'a, 'b> {
    Action {
        action_name: &'b str,
        parameters: &'a [Parameter],
    },
    Entity {
        semantic_key: &'b str,
        entity: &'a Entity,
    },
    Scenario {
        instances: &'b HashMap<&'a str, &'a Entity>,
    },
}

enum OperandShape<'a> {
    Literal,
    Typed(ResolvedType<'a>),
    Unbound,
}

fn project_model(document: &Document) -> Result<CheckedModel, ProtocolBuildError> {
    let system = document
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::System(system) => Some(system),
            _ => None,
        })
        .ok_or_else(|| projection_error("checked document has no top-level system"))?;
    let mut actions = HashMap::new();
    collect_actions(&document.declarations, &mut actions);
    let mut projector = Projector {
        index: semantic::build_type_index(document),
        actions,
        next_node: 0,
    };
    let node_id = projector.node_id();
    let mut declarations = Vec::new();
    let mut container_path = Vec::new();
    projector.declarations(&system.declarations, &mut container_path, &mut declarations)?;
    Ok(CheckedModel {
        system: SystemNode {
            node_id,
            semantic_key: format!("system:{}", system.name.text),
            name: system.name.text.clone(),
            shell_locations: vec![source_location(system.span)],
            declarations,
        },
    })
}

fn collect_actions<'a>(
    declarations: &'a [Declaration],
    actions: &mut HashMap<&'a str, &'a Action>,
) {
    for declaration in declarations {
        if let Declaration::Action(action) = declaration {
            actions.insert(action.name.text.as_str(), action);
        }
        collect_actions(declaration.declarations(), actions);
    }
}

impl<'a> Projector<'a> {
    fn declarations(
        &mut self,
        declarations: &'a [Declaration],
        container_path: &mut Vec<ContainerRef>,
        output: &mut Vec<DeclarationNode>,
    ) -> Result<(), ProtocolBuildError> {
        for declaration in declarations {
            match declaration {
                Declaration::Container(container) => {
                    container_path.push(ContainerRef {
                        container_kind: container.kind.clone(),
                        name: container.name.text.clone(),
                        location: source_location(container.name.span),
                    });
                    self.declarations(&container.declarations, container_path, output)?;
                    container_path.pop();
                }
                Declaration::Enum(enumeration) => {
                    output.push(DeclarationNode::Enum(
                        self.enum_node(enumeration, container_path)?,
                    ));
                }
                Declaration::Entity(entity) => {
                    output.push(DeclarationNode::Entity(
                        self.entity_node(entity, container_path)?,
                    ));
                }
                Declaration::Action(action) => {
                    output.push(DeclarationNode::Action(
                        self.action_node(action, container_path)?,
                    ));
                }
                Declaration::Scenario(scenario) => {
                    output.push(DeclarationNode::Scenario(
                        self.scenario_node(scenario, container_path)?,
                    ));
                }
                Declaration::System(_) => {
                    return Err(projection_error("nested system in a checked document"));
                }
            }
        }
        Ok(())
    }

    fn enum_node(
        &mut self,
        enumeration: &'a Enum,
        container_path: &[ContainerRef],
    ) -> Result<EnumNode, ProtocolBuildError> {
        let semantic_key = format!("enum:{}", enumeration.name.text);
        let node_id = self.node_id();
        let members = enumeration
            .members
            .iter()
            .map(|member| EnumMemberNode {
                node_id: self.node_id(),
                semantic_key: format!("enum_member:{}.{}", enumeration.name.text, member.text),
                name: member.text.clone(),
                location: source_location(member.span),
                enum_semantic_key: semantic_key.clone(),
            })
            .collect();
        Ok(EnumNode {
            node_id,
            semantic_key,
            name: enumeration.name.text.clone(),
            location: source_location(enumeration.span),
            container_path: container_path.to_vec(),
            members,
        })
    }

    fn entity_node(
        &mut self,
        entity: &'a Entity,
        container_path: &[ContainerRef],
    ) -> Result<EntityNode, ProtocolBuildError> {
        let semantic_key = format!("entity:{}", entity.name.text);
        let node_id = self.node_id();
        let mut fields = Vec::new();
        for field in &entity.fields {
            let resolved = self.named_type(&field.type_name)?;
            fields.push(FieldNode {
                node_id: self.node_id(),
                semantic_key: format!("field:{}.{}", entity.name.text, field.name.text),
                name: field.name.text.clone(),
                location: source_location(field.span),
                field_type: self.type_ref(resolved),
            });
        }
        let scope = ExpressionScope::Entity {
            semantic_key: &semantic_key,
            entity,
        };
        let mut invariants = Vec::new();
        for invariant in &entity.invariants {
            invariants.push(self.expression(invariant, None, scope)?);
        }
        Ok(EntityNode {
            node_id,
            semantic_key,
            name: entity.name.text.clone(),
            location: source_location(entity.span),
            container_path: container_path.to_vec(),
            fields,
            invariants,
        })
    }

    fn action_node(
        &mut self,
        action: &'a Action,
        container_path: &[ContainerRef],
    ) -> Result<ActionNode, ProtocolBuildError> {
        let semantic_key = format!("action:{}", action.name.text);
        let node_id = self.node_id();
        let mut parameters = Vec::new();
        for parameter in &action.parameters {
            let resolved = self.named_type(&parameter.type_name)?;
            parameters.push(ParameterNode {
                node_id: self.node_id(),
                semantic_key: format!("parameter:{}.{}", action.name.text, parameter.name.text),
                name: parameter.name.text.clone(),
                location: source_location(parameter.span),
                action_semantic_key: semantic_key.clone(),
                parameter_type: self.type_ref(resolved),
            });
        }
        let scope = ExpressionScope::Action {
            action_name: &action.name.text,
            parameters: &action.parameters,
        };
        let mut clauses = Vec::new();
        for clause in &action.clauses {
            let (clause_kind, state_phase) = match clause.kind {
                ClauseKind::Requires => ("requires", "pre"),
                ClauseKind::Effects => ("effects", "effect"),
                ClauseKind::Ensures => ("ensures", "post"),
                ClauseKind::Invariant => ("invariant", "both"),
            };
            let mut expressions = Vec::new();
            for expression in &clause.expressions {
                expressions.push(match expression {
                    ClauseExpression::Predicate(expression) => {
                        ClauseExpressionNode::Predicate(self.expression(expression, None, scope)?)
                    }
                    ClauseExpression::Assignment(assignment) => ClauseExpressionNode::Assignment(
                        Box::new(self.assignment(assignment, scope)?),
                    ),
                });
            }
            clauses.push(ClauseNode {
                clause_kind,
                state_phase,
                expressions,
                location: source_location(clause.span),
            });
        }
        Ok(ActionNode {
            node_id,
            semantic_key,
            name: action.name.text.clone(),
            location: source_location(action.span),
            container_path: container_path.to_vec(),
            parameters,
            clauses,
        })
    }

    fn scenario_node(
        &mut self,
        scenario: &'a Scenario,
        container_path: &[ContainerRef],
    ) -> Result<ScenarioNode, ProtocolBuildError> {
        let run = scenario
            .items
            .iter()
            .find_map(|item| match item {
                ScenarioItem::Run(run) => Some(run),
                _ => None,
            })
            .ok_or_else(|| projection_error("checked scenario has no run item"))?;
        let action = *self
            .actions
            .get(run.action.text.as_str())
            .ok_or_else(|| projection_error(format!("unresolved action '{}'", run.action.text)))?;
        let mut instances: HashMap<&'a str, &'a Entity> = HashMap::new();
        let mut arguments = Vec::new();
        for (argument, parameter) in run.arguments.iter().zip(&action.parameters) {
            let resolved = self.named_type(&parameter.type_name)?;
            let ResolvedType::Entity(entity) = resolved else {
                return Err(projection_error(format!(
                    "run argument '{}' does not bind an entity parameter",
                    argument.text
                )));
            };
            instances.insert(argument.text.as_str(), entity);
            arguments.push(RunArgumentNode {
                name: argument.text.clone(),
                parameter_semantic_key: format!(
                    "parameter:{}.{}",
                    action.name.text, parameter.name.text
                ),
                argument_type: self.type_ref(resolved),
                location: source_location(argument.span),
            });
        }
        let scope = ExpressionScope::Scenario {
            instances: &instances,
        };
        let mut items = Vec::new();
        for item in &scenario.items {
            items.push(match item {
                ScenarioItem::Given(assignment) => ScenarioItemNode::Given {
                    assignment: Box::new(self.assignment(assignment, scope)?),
                    location: source_location(assignment.span),
                },
                ScenarioItem::Run(run) => ScenarioItemNode::Run {
                    action_semantic_key: format!("action:{}", action.name.text),
                    arguments: arguments.clone(),
                    location: source_location(run.span),
                },
                ScenarioItem::Expect(expression) => ScenarioItemNode::Expect {
                    expression: self.expression(expression, None, scope)?,
                    location: source_location(expression.span),
                },
            });
        }
        Ok(ScenarioNode {
            node_id: self.node_id(),
            semantic_key: format!("scenario:{}", scenario.name.text),
            name: scenario.name.text.clone(),
            location: source_location(scenario.span),
            container_path: container_path.to_vec(),
            items,
        })
    }

    fn assignment(
        &mut self,
        assignment: &Assignment,
        scope: ExpressionScope<'a, '_>,
    ) -> Result<AssignmentNode, ProtocolBuildError> {
        let target = self.path_expression(&assignment.target, assignment.target.span, scope)?;
        let target_type = target.resolved_type.clone();
        let expected = self.resolved_of_type_ref(&target_type);
        let operator = match assignment.operator {
            AssignmentOperator::Set => "set",
            AssignmentOperator::Add => "add",
            AssignmentOperator::Subtract => "subtract",
        };
        let value = self.expression(&assignment.value, expected, scope)?;
        Ok(AssignmentNode {
            target,
            operator,
            value,
            target_type,
            location: source_location(assignment.span),
        })
    }

    /// Reverse lookup used only to thread an expected type into contextual
    /// literal typing; the authoritative resolution already happened.
    fn resolved_of_type_ref(&self, type_ref: &TypeRef) -> Option<ResolvedType<'a>> {
        match type_ref {
            TypeRef::Builtin { name } => {
                let canonical = if *name == "Id" { "ID" } else { name };
                semantic::resolve_type(
                    &Name {
                        text: canonical.to_owned(),
                        span: Span::default(),
                    },
                    &self.index,
                )
            }
            TypeRef::Enum { semantic_key } | TypeRef::Entity { semantic_key } => {
                let name = semantic_key.split(':').nth(1)?;
                semantic::resolve_type(
                    &Name {
                        text: name.to_owned(),
                        span: Span::default(),
                    },
                    &self.index,
                )
            }
        }
    }

    fn expression(
        &mut self,
        expression: &Expr,
        expected: Option<ResolvedType<'a>>,
        scope: ExpressionScope<'a, '_>,
    ) -> Result<ExpressionNode, ProtocolBuildError> {
        match &expression.kind {
            ExprKind::Integer(value) => {
                let decimal = matches!(
                    expected,
                    Some(ResolvedType::Builtin(semantic::BuiltinType::Decimal))
                );
                Ok(ExpressionNode::Integer {
                    value: value.to_string(),
                    resolved_type: TypeRef::Builtin {
                        name: if decimal { "Decimal" } else { "Integer" },
                    },
                    location: source_location(expression.span),
                })
            }
            ExprKind::Boolean(value) => Ok(ExpressionNode::Boolean {
                value: *value,
                resolved_type: TypeRef::Builtin { name: "Boolean" },
                location: source_location(expression.span),
            }),
            ExprKind::Path(path) => {
                if let Some(ResolvedType::Enum(enumeration)) = expected
                    && self.is_unbound_member(path, enumeration, scope)
                {
                    return Ok(self.enum_member_expression(path, enumeration, expression.span));
                }
                Ok(ExpressionNode::Path(self.path_expression(
                    path,
                    expression.span,
                    scope,
                )?))
            }
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                let left_shape = self.operand_shape(left, scope);
                let right_shape = self.operand_shape(right, scope);
                let left_expected = self.comparison_expected(&right_shape);
                let right_expected = self.comparison_expected(&left_shape);
                let left = self.expression(left, left_expected, scope)?;
                let right = self.expression(right, right_expected, scope)?;
                Ok(ExpressionNode::Binary {
                    operator: match operator {
                        BinaryOperator::Equal => "equal",
                        BinaryOperator::NotEqual => "not_equal",
                        BinaryOperator::Greater => "greater",
                        BinaryOperator::GreaterEqual => "greater_equal",
                        BinaryOperator::Less => "less",
                        BinaryOperator::LessEqual => "less_equal",
                    },
                    left: Box::new(left),
                    right: Box::new(right),
                    resolved_type: TypeRef::Builtin { name: "Boolean" },
                    location: source_location(expression.span),
                })
            }
            ExprKind::Not(operand) => Ok(ExpressionNode::Not {
                operand: Box::new(self.expression(operand, None, scope)?),
                resolved_type: TypeRef::Builtin { name: "Boolean" },
                location: source_location(expression.span),
            }),
            ExprKind::Or { left, right } => Ok(ExpressionNode::Or {
                left: Box::new(self.expression(left, None, scope)?),
                right: Box::new(self.expression(right, None, scope)?),
                resolved_type: TypeRef::Builtin { name: "Boolean" },
                location: source_location(expression.span),
            }),
        }
    }

    fn comparison_expected(&self, other: &OperandShape<'a>) -> Option<ResolvedType<'a>> {
        match other {
            OperandShape::Typed(resolved) => Some(*resolved),
            OperandShape::Literal | OperandShape::Unbound => None,
        }
    }

    fn operand_shape(&self, expression: &Expr, scope: ExpressionScope<'a, '_>) -> OperandShape<'a> {
        match &expression.kind {
            ExprKind::Integer(_) | ExprKind::Boolean(_) => OperandShape::Literal,
            ExprKind::Path(path) => match self.path_types(path, scope) {
                Some((_, types)) => match types.last() {
                    Some(resolved) => OperandShape::Typed(*resolved),
                    None => OperandShape::Unbound,
                },
                None => OperandShape::Unbound,
            },
            ExprKind::Binary { .. } | ExprKind::Not(_) | ExprKind::Or { .. } => {
                OperandShape::Typed(ResolvedType::Builtin(semantic::BuiltinType::Boolean))
            }
        }
    }

    fn is_unbound_member(
        &self,
        path: &Path,
        enumeration: &Enum,
        scope: ExpressionScope<'a, '_>,
    ) -> bool {
        path.segments.len() == 1
            && self.path_types(path, scope).is_none()
            && enumeration
                .members
                .iter()
                .any(|member| member.text == path.segments[0].text)
    }

    fn enum_member_expression(
        &self,
        path: &Path,
        enumeration: &Enum,
        span: Span,
    ) -> ExpressionNode {
        let member = &path.segments[0].text;
        ExpressionNode::EnumMember {
            enum_semantic_key: format!("enum:{}", enumeration.name.text),
            member_semantic_key: format!("enum_member:{}.{member}", enumeration.name.text),
            member: member.clone(),
            resolved_type: TypeRef::Enum {
                semantic_key: format!("enum:{}", enumeration.name.text),
            },
            location: source_location(span),
        }
    }

    /// Root binding plus one resolved type per source segment, or None when
    /// the first segment does not bind in this scope.
    fn path_types(
        &self,
        path: &Path,
        scope: ExpressionScope<'a, '_>,
    ) -> Option<(PathRootNode, Vec<ResolvedType<'a>>)> {
        let root_name = path.segments[0].text.as_str();
        let (root, mut current) = match scope {
            ExpressionScope::Action {
                action_name,
                parameters,
            } => {
                let parameter = parameters
                    .iter()
                    .find(|parameter| parameter.name.text == root_name)?;
                (
                    PathRootNode::ActionParameter {
                        semantic_key: format!("parameter:{action_name}.{root_name}"),
                    },
                    semantic::resolve_type(&parameter.type_name, &self.index)?,
                )
            }
            ExpressionScope::Entity {
                semantic_key,
                entity,
            } => {
                let field = entity
                    .fields
                    .iter()
                    .find(|field| field.name.text == root_name)?;
                (
                    PathRootNode::EntitySelf {
                        semantic_key: semantic_key.to_owned(),
                    },
                    semantic::resolve_type(&field.type_name, &self.index)?,
                )
            }
            ExpressionScope::Scenario { instances } => {
                let entity = instances.get(root_name)?;
                (
                    PathRootNode::ScenarioInstance {
                        name: root_name.to_owned(),
                        instance_type: TypeRef::Entity {
                            semantic_key: format!("entity:{}", entity.name.text),
                        },
                    },
                    ResolvedType::Entity(entity),
                )
            }
        };
        let mut types = vec![current];
        for segment in &path.segments[1..] {
            let ResolvedType::Entity(entity) = current else {
                return None;
            };
            let field = entity
                .fields
                .iter()
                .find(|field| field.name.text == segment.text)?;
            current = semantic::resolve_type(&field.type_name, &self.index)?;
            types.push(current);
        }
        Some((root, types))
    }

    fn path_expression(
        &mut self,
        path: &Path,
        span: Span,
        scope: ExpressionScope<'a, '_>,
    ) -> Result<PathExpressionNode, ProtocolBuildError> {
        let (root, types) = self
            .path_types(path, scope)
            .ok_or_else(|| projection_error(format!("unresolved path '{}'", path.display())))?;
        let segments = path
            .segments
            .iter()
            .zip(&types)
            .map(|(segment, resolved)| PathSegmentNode {
                name: segment.text.clone(),
                resolved_type: self.type_ref(*resolved),
                location: source_location(segment.span),
            })
            .collect::<Vec<_>>();
        let resolved_type = self.type_ref(*types.last().expect("non-empty path types"));
        Ok(PathExpressionNode {
            root,
            segments,
            resolved_type,
            location: source_location(span),
        })
    }
}

// ---------------------------------------------------------------------------
// Invariant validation
// ---------------------------------------------------------------------------

impl CheckedSemanticsDocument {
    pub fn validate(&self) -> Result<(), ProtocolInvariantError> {
        self.validate_envelope()?;
        self.validate_locations()?;
        self.validate_status()?;
        self.validate_coverage()?;
        if let Some(model) = &self.result.checked_model {
            self.validate_model(model)?;
        }
        Ok(())
    }

    fn validate_envelope(&self) -> Result<(), ProtocolInvariantError> {
        if self.producer_version.is_empty() {
            return Err(ProtocolInvariantError::Envelope(
                "producer version must not be empty".to_owned(),
            ));
        }
        if self.sources.len() != 1 {
            return Err(ProtocolInvariantError::Source(format!(
                "a file subject requires exactly one source, found {}",
                self.sources.len()
            )));
        }
        let source = &self.sources[0];
        if source.source_id != "source:0" {
            return Err(ProtocolInvariantError::Source(format!(
                "single-file source id must be 'source:0', found '{}'",
                source.source_id
            )));
        }
        for name in [&self.subject.name, &source.name] {
            if name.is_empty() || name.contains('/') || name.contains('\\') {
                return Err(ProtocolInvariantError::Source(format!(
                    "invalid logical source name {name:?}"
                )));
            }
        }
        if self.subject.name != source.name {
            return Err(ProtocolInvariantError::Source(
                "file subject name must equal its source name".to_owned(),
            ));
        }
        let recomputed = sha256::hex_digest(source.content.as_bytes());
        if source.revision.value != recomputed {
            return Err(ProtocolInvariantError::Source(format!(
                "source revision digest mismatch: recorded {}, recomputed {recomputed}",
                source.revision.value
            )));
        }
        if self.subject.revision != source.revision {
            return Err(ProtocolInvariantError::Source(
                "file subject revision must equal its source revision".to_owned(),
            ));
        }
        Ok(())
    }

    fn validate_locations(&self) -> Result<(), ProtocolInvariantError> {
        let mut locations = Vec::new();
        for finding in &self.result.findings {
            locations.push(&finding.primary_location);
        }
        for item in &self.result.coverage.unmodeled {
            locations.push(&item.location);
        }
        if let Some(model) = &self.result.checked_model {
            collect_model_locations(model, &mut locations);
        }
        let content = &self.sources[0].content;
        for location in locations {
            match location {
                ProtocolLocation::Subject => {
                    return Err(ProtocolInvariantError::Location(
                        "subject locations are reserved for source-less project findings"
                            .to_owned(),
                    ));
                }
                ProtocolLocation::Source {
                    source_id,
                    byte_range,
                } => {
                    if source_id != "source:0" {
                        return Err(ProtocolInvariantError::Location(format!(
                            "location references unknown source '{source_id}'"
                        )));
                    }
                    if byte_range.start > byte_range.end
                        || byte_range.end > content.len()
                        || !content.is_char_boundary(byte_range.start)
                        || !content.is_char_boundary(byte_range.end)
                    {
                        return Err(ProtocolInvariantError::Location(format!(
                            "byte range {}..{} is not a valid UTF-8 range of the source",
                            byte_range.start, byte_range.end
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_status(&self) -> Result<(), ProtocolInvariantError> {
        for (index, finding) in self.result.findings.iter().enumerate() {
            let expected = format!("finding:{index}");
            if finding.finding_id != expected {
                return Err(ProtocolInvariantError::Identity(format!(
                    "finding ids must be consecutive ordinals; expected '{expected}', found '{}'",
                    finding.finding_id
                )));
            }
            let is_coverage = finding.category == FindingCategory::Coverage;
            if is_coverage != finding.details.is_some() {
                return Err(ProtocolInvariantError::Coverage(format!(
                    "typed details are required exactly for coverage findings ('{}')",
                    finding.finding_id
                )));
            }
            if is_coverage && finding.severity != Severity::Warning {
                return Err(ProtocolInvariantError::Coverage(format!(
                    "coverage findings must be warnings ('{}')",
                    finding.finding_id
                )));
            }
        }
        let has_error = self
            .result
            .findings
            .iter()
            .any(|finding| finding.severity == Severity::Error);
        match self.result.status {
            ResultStatus::Invalid => {
                if !has_error {
                    return Err(ProtocolInvariantError::Status(
                        "an invalid result requires at least one error finding".to_owned(),
                    ));
                }
                if self.result.checked_model.is_some() {
                    return Err(ProtocolInvariantError::Status(
                        "an invalid result must not carry a checked model".to_owned(),
                    ));
                }
            }
            ResultStatus::Valid => {
                if has_error {
                    return Err(ProtocolInvariantError::Status(
                        "a valid result must not contain error findings".to_owned(),
                    ));
                }
                if self.result.checked_model.is_none() {
                    return Err(ProtocolInvariantError::Status(
                        "a valid result requires a checked model".to_owned(),
                    ));
                }
                if self.result.coverage.assessment != CoverageAssessment::Complete {
                    return Err(ProtocolInvariantError::Status(
                        "a valid result requires a complete coverage assessment".to_owned(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_coverage(&self) -> Result<(), ProtocolInvariantError> {
        let coverage = &self.result.coverage;
        match coverage.assessment {
            CoverageAssessment::Unavailable => {
                if coverage.fully_modeled.is_some() || !coverage.unmodeled.is_empty() {
                    return Err(ProtocolInvariantError::Coverage(
                        "an unavailable assessment requires null fully_modeled and no unmodeled entries"
                            .to_owned(),
                    ));
                }
            }
            CoverageAssessment::Complete => {
                let Some(fully_modeled) = coverage.fully_modeled else {
                    return Err(ProtocolInvariantError::Coverage(
                        "a complete assessment requires a boolean fully_modeled".to_owned(),
                    ));
                };
                if fully_modeled != coverage.unmodeled.is_empty() {
                    return Err(ProtocolInvariantError::Coverage(
                        "fully_modeled must be true exactly when no unmodeled entries exist"
                            .to_owned(),
                    ));
                }
            }
        }
        let coverage_findings: Vec<&Finding> = self
            .result
            .findings
            .iter()
            .filter(|finding| finding.category == FindingCategory::Coverage)
            .collect();
        if coverage_findings.len() != coverage.unmodeled.len() {
            return Err(ProtocolInvariantError::Coverage(format!(
                "{} unmodeled entries but {} coverage warnings",
                coverage.unmodeled.len(),
                coverage_findings.len()
            )));
        }
        for item in &coverage.unmodeled {
            let matches = coverage_findings
                .iter()
                .filter(|finding| {
                    finding.details.as_ref() == Some(&item.details)
                        && finding.primary_location == item.location
                })
                .count();
            if matches != 1 {
                return Err(ProtocolInvariantError::Coverage(
                    "every unmodeled entry requires exactly one matching coverage warning"
                        .to_owned(),
                ));
            }
        }
        Ok(())
    }

    fn validate_model(&self, model: &CheckedModel) -> Result<(), ProtocolInvariantError> {
        let mut keys = ModelKeys::default();
        collect_model_keys(model, &mut keys)?;
        check_model_references(model, &keys)?;
        self.validate_model_completeness(model)
    }

    fn validate_model_completeness(
        &self,
        model: &CheckedModel,
    ) -> Result<(), ProtocolInvariantError> {
        let source = &self.sources[0].content;
        let document = crate::parser::parse(source).map_err(|_| {
            ProtocolInvariantError::Model(
                "checked model present but the recorded source no longer parses".to_owned(),
            )
        })?;
        let system = document
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::System(system) => Some(system),
                _ => None,
            })
            .ok_or_else(|| {
                ProtocolInvariantError::Model("recorded source has no top-level system".to_owned())
            })?;
        if system.name.text != model.system.name {
            return Err(ProtocolInvariantError::Model(format!(
                "model system '{}' does not match source system '{}'",
                model.system.name, system.name.text
            )));
        }
        if model.system.shell_locations != vec![source_location(system.span)] {
            return Err(ProtocolInvariantError::Model(
                "system shell locations do not match the source system span".to_owned(),
            ));
        }
        let mut expected = Vec::new();
        collect_source_declarations(&system.declarations, &mut expected);
        let actual: Vec<(&'static str, String)> = model
            .system
            .declarations
            .iter()
            .map(|declaration| match declaration {
                DeclarationNode::Enum(node) => ("enum", node.name.clone()),
                DeclarationNode::Entity(node) => ("entity", node.name.clone()),
                DeclarationNode::Action(node) => ("action", node.name.clone()),
                DeclarationNode::Scenario(node) => ("scenario", node.name.clone()),
            })
            .collect();
        if expected != actual {
            return Err(ProtocolInvariantError::Model(format!(
                "model declarations {actual:?} do not cover the source declarations {expected:?}"
            )));
        }
        Ok(())
    }
}

fn collect_source_declarations(
    declarations: &[Declaration],
    output: &mut Vec<(&'static str, String)>,
) {
    for declaration in declarations {
        match declaration {
            Declaration::Container(container) => {
                collect_source_declarations(&container.declarations, output)
            }
            Declaration::Enum(item) => output.push(("enum", item.name.text.clone())),
            Declaration::Entity(item) => output.push(("entity", item.name.text.clone())),
            Declaration::Action(item) => output.push(("action", item.name.text.clone())),
            Declaration::Scenario(item) => output.push(("scenario", item.name.text.clone())),
            Declaration::System(_) => {}
        }
    }
}

#[derive(Default)]
struct ModelKeys {
    node_ids: std::collections::HashSet<String>,
    semantic_keys: std::collections::HashSet<String>,
    enums: std::collections::HashSet<String>,
    entities: std::collections::HashSet<String>,
    actions: std::collections::HashSet<String>,
    parameters: std::collections::HashSet<String>,
    members: std::collections::HashSet<String>,
}

impl ModelKeys {
    fn node(&mut self, node_id: &str) -> Result<(), ProtocolInvariantError> {
        if !node_id.starts_with("node:") || !self.node_ids.insert(node_id.to_owned()) {
            return Err(ProtocolInvariantError::Identity(format!(
                "node id '{node_id}' is malformed or duplicated"
            )));
        }
        Ok(())
    }

    fn key(&mut self, semantic_key: &str) -> Result<(), ProtocolInvariantError> {
        if semantic_key.is_empty() || !self.semantic_keys.insert(semantic_key.to_owned()) {
            return Err(ProtocolInvariantError::Identity(format!(
                "semantic key '{semantic_key}' is empty or duplicated"
            )));
        }
        Ok(())
    }
}

fn collect_model_keys(
    model: &CheckedModel,
    keys: &mut ModelKeys,
) -> Result<(), ProtocolInvariantError> {
    keys.node(&model.system.node_id)?;
    keys.key(&model.system.semantic_key)?;
    for declaration in &model.system.declarations {
        match declaration {
            DeclarationNode::Enum(node) => {
                keys.node(&node.node_id)?;
                keys.key(&node.semantic_key)?;
                keys.enums.insert(node.semantic_key.clone());
                for member in &node.members {
                    keys.node(&member.node_id)?;
                    keys.key(&member.semantic_key)?;
                    keys.members.insert(member.semantic_key.clone());
                }
            }
            DeclarationNode::Entity(node) => {
                keys.node(&node.node_id)?;
                keys.key(&node.semantic_key)?;
                keys.entities.insert(node.semantic_key.clone());
                for field in &node.fields {
                    keys.node(&field.node_id)?;
                    keys.key(&field.semantic_key)?;
                }
            }
            DeclarationNode::Action(node) => {
                keys.node(&node.node_id)?;
                keys.key(&node.semantic_key)?;
                keys.actions.insert(node.semantic_key.clone());
                for parameter in &node.parameters {
                    keys.node(&parameter.node_id)?;
                    keys.key(&parameter.semantic_key)?;
                    keys.parameters.insert(parameter.semantic_key.clone());
                }
            }
            DeclarationNode::Scenario(node) => {
                keys.node(&node.node_id)?;
                keys.key(&node.semantic_key)?;
            }
        }
    }
    Ok(())
}

fn reference_error(message: String) -> ProtocolInvariantError {
    ProtocolInvariantError::Reference(message)
}

fn check_type_ref(type_ref: &TypeRef, keys: &ModelKeys) -> Result<(), ProtocolInvariantError> {
    match type_ref {
        TypeRef::Builtin { name } => {
            if !["Boolean", "Integer", "Decimal", "String", "Id"].contains(name) {
                return Err(reference_error(format!("unknown builtin type '{name}'")));
            }
        }
        TypeRef::Enum { semantic_key } => {
            if !keys.enums.contains(semantic_key) {
                return Err(reference_error(format!(
                    "dangling enum reference '{semantic_key}'"
                )));
            }
        }
        TypeRef::Entity { semantic_key } => {
            if !keys.entities.contains(semantic_key) {
                return Err(reference_error(format!(
                    "dangling entity reference '{semantic_key}'"
                )));
            }
        }
    }
    Ok(())
}

fn require_boolean(type_ref: &TypeRef, context: &str) -> Result<(), ProtocolInvariantError> {
    if !matches!(type_ref, TypeRef::Builtin { name: "Boolean" }) {
        return Err(ProtocolInvariantError::Model(format!(
            "{context} must resolve to Boolean"
        )));
    }
    Ok(())
}

fn check_expression(
    expression: &ExpressionNode,
    keys: &ModelKeys,
) -> Result<(), ProtocolInvariantError> {
    match expression {
        ExpressionNode::Integer {
            value,
            resolved_type,
            ..
        } => {
            if value.parse::<i64>().is_err() {
                return Err(ProtocolInvariantError::Model(format!(
                    "integer literal '{value}' is outside the signed 64-bit range"
                )));
            }
            if !matches!(
                resolved_type,
                TypeRef::Builtin {
                    name: "Integer" | "Decimal"
                }
            ) {
                return Err(ProtocolInvariantError::Model(
                    "integer literals must resolve to Integer or Decimal".to_owned(),
                ));
            }
        }
        ExpressionNode::Boolean { resolved_type, .. } => {
            require_boolean(resolved_type, "a Boolean literal")?;
        }
        ExpressionNode::EnumMember {
            enum_semantic_key,
            member_semantic_key,
            resolved_type,
            ..
        } => {
            if !keys.enums.contains(enum_semantic_key) {
                return Err(reference_error(format!(
                    "dangling enum reference '{enum_semantic_key}'"
                )));
            }
            if !keys.members.contains(member_semantic_key) {
                return Err(reference_error(format!(
                    "dangling enum member reference '{member_semantic_key}'"
                )));
            }
            check_type_ref(resolved_type, keys)?;
        }
        ExpressionNode::Path(path) => check_path(path, keys)?,
        ExpressionNode::Binary {
            left,
            right,
            resolved_type,
            ..
        } => {
            check_expression(left, keys)?;
            check_expression(right, keys)?;
            require_boolean(resolved_type, "a comparison")?;
        }
        ExpressionNode::Not {
            operand,
            resolved_type,
            ..
        } => {
            check_expression(operand, keys)?;
            require_boolean(resolved_type, "a negation")?;
        }
        ExpressionNode::Or {
            left,
            right,
            resolved_type,
            ..
        } => {
            check_expression(left, keys)?;
            check_expression(right, keys)?;
            require_boolean(resolved_type, "a disjunction")?;
        }
    }
    Ok(())
}

fn check_path(path: &PathExpressionNode, keys: &ModelKeys) -> Result<(), ProtocolInvariantError> {
    match &path.root {
        PathRootNode::ActionParameter { semantic_key } => {
            if !keys.parameters.contains(semantic_key) {
                return Err(reference_error(format!(
                    "dangling parameter reference '{semantic_key}'"
                )));
            }
        }
        PathRootNode::EntitySelf { semantic_key } => {
            if !keys.entities.contains(semantic_key) {
                return Err(reference_error(format!(
                    "dangling entity reference '{semantic_key}'"
                )));
            }
        }
        PathRootNode::ScenarioInstance { instance_type, .. } => {
            check_type_ref(instance_type, keys)?;
        }
    }
    if path.segments.is_empty() {
        return Err(ProtocolInvariantError::Model(
            "path expressions require at least one segment".to_owned(),
        ));
    }
    for segment in &path.segments {
        check_type_ref(&segment.resolved_type, keys)?;
    }
    check_type_ref(&path.resolved_type, keys)
}

fn check_assignment_node(
    assignment: &AssignmentNode,
    keys: &ModelKeys,
) -> Result<(), ProtocolInvariantError> {
    check_path(&assignment.target, keys)?;
    check_type_ref(&assignment.target_type, keys)?;
    if !["set", "add", "subtract"].contains(&assignment.operator) {
        return Err(ProtocolInvariantError::Model(format!(
            "unknown assignment operator '{}'",
            assignment.operator
        )));
    }
    check_expression(&assignment.value, keys)
}

fn check_model_references(
    model: &CheckedModel,
    keys: &ModelKeys,
) -> Result<(), ProtocolInvariantError> {
    for declaration in &model.system.declarations {
        match declaration {
            DeclarationNode::Enum(node) => {
                for member in &node.members {
                    if !keys.enums.contains(&member.enum_semantic_key) {
                        return Err(reference_error(format!(
                            "dangling enum reference '{}'",
                            member.enum_semantic_key
                        )));
                    }
                }
            }
            DeclarationNode::Entity(node) => {
                for field in &node.fields {
                    check_type_ref(&field.field_type, keys)?;
                }
                for invariant in &node.invariants {
                    check_expression(invariant, keys)?;
                }
            }
            DeclarationNode::Action(node) => {
                for parameter in &node.parameters {
                    if !keys.actions.contains(&parameter.action_semantic_key) {
                        return Err(reference_error(format!(
                            "dangling action reference '{}'",
                            parameter.action_semantic_key
                        )));
                    }
                    check_type_ref(&parameter.parameter_type, keys)?;
                }
                for clause in &node.clauses {
                    let expected_phase = match clause.clause_kind {
                        "requires" => "pre",
                        "effects" => "effect",
                        "ensures" => "post",
                        "invariant" => "both",
                        other => {
                            return Err(ProtocolInvariantError::Model(format!(
                                "unknown clause kind '{other}'"
                            )));
                        }
                    };
                    if clause.state_phase != expected_phase {
                        return Err(ProtocolInvariantError::Model(format!(
                            "clause kind '{}' requires state phase '{expected_phase}'",
                            clause.clause_kind
                        )));
                    }
                    if clause.expressions.is_empty() {
                        return Err(ProtocolInvariantError::Model(
                            "clauses require at least one expression".to_owned(),
                        ));
                    }
                    for expression in &clause.expressions {
                        match expression {
                            ClauseExpressionNode::Predicate(expression) => {
                                if clause.clause_kind == "effects" {
                                    return Err(ProtocolInvariantError::Model(
                                        "effects clauses carry assignments only".to_owned(),
                                    ));
                                }
                                check_expression(expression, keys)?;
                            }
                            ClauseExpressionNode::Assignment(assignment) => {
                                if clause.clause_kind != "effects" {
                                    return Err(ProtocolInvariantError::Model(
                                        "assignments occur only in effects clauses".to_owned(),
                                    ));
                                }
                                check_assignment_node(assignment, keys)?;
                            }
                        }
                    }
                }
            }
            DeclarationNode::Scenario(node) => {
                for item in &node.items {
                    match item {
                        ScenarioItemNode::Given { assignment, .. } => {
                            check_assignment_node(assignment, keys)?;
                        }
                        ScenarioItemNode::Run {
                            action_semantic_key,
                            arguments,
                            ..
                        } => {
                            if !keys.actions.contains(action_semantic_key) {
                                return Err(reference_error(format!(
                                    "dangling action reference '{action_semantic_key}'"
                                )));
                            }
                            for argument in arguments {
                                if !keys.parameters.contains(&argument.parameter_semantic_key) {
                                    return Err(reference_error(format!(
                                        "dangling parameter reference '{}'",
                                        argument.parameter_semantic_key
                                    )));
                                }
                                check_type_ref(&argument.argument_type, keys)?;
                            }
                        }
                        ScenarioItemNode::Expect { expression, .. } => {
                            check_expression(expression, keys)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn collect_model_locations<'a>(model: &'a CheckedModel, locations: &mut Vec<&'a ProtocolLocation>) {
    fn expression<'a>(node: &'a ExpressionNode, locations: &mut Vec<&'a ProtocolLocation>) {
        match node {
            ExpressionNode::Integer { location, .. }
            | ExpressionNode::Boolean { location, .. }
            | ExpressionNode::EnumMember { location, .. } => locations.push(location),
            ExpressionNode::Path(path) => path_locations(path, locations),
            ExpressionNode::Binary {
                left,
                right,
                location,
                ..
            }
            | ExpressionNode::Or {
                left,
                right,
                location,
                ..
            } => {
                expression(left, locations);
                expression(right, locations);
                locations.push(location);
            }
            ExpressionNode::Not {
                operand, location, ..
            } => {
                expression(operand, locations);
                locations.push(location);
            }
        }
    }
    fn path_locations<'a>(path: &'a PathExpressionNode, locations: &mut Vec<&'a ProtocolLocation>) {
        for segment in &path.segments {
            locations.push(&segment.location);
        }
        locations.push(&path.location);
    }
    fn assignment<'a>(node: &'a AssignmentNode, locations: &mut Vec<&'a ProtocolLocation>) {
        path_locations(&node.target, locations);
        expression(&node.value, locations);
        locations.push(&node.location);
    }
    for shell in &model.system.shell_locations {
        locations.push(shell);
    }
    for declaration in &model.system.declarations {
        match declaration {
            DeclarationNode::Enum(node) => {
                locations.push(&node.location);
                for container in &node.container_path {
                    locations.push(&container.location);
                }
                for member in &node.members {
                    locations.push(&member.location);
                }
            }
            DeclarationNode::Entity(node) => {
                locations.push(&node.location);
                for container in &node.container_path {
                    locations.push(&container.location);
                }
                for field in &node.fields {
                    locations.push(&field.location);
                }
                for invariant in &node.invariants {
                    expression(invariant, locations);
                }
            }
            DeclarationNode::Action(node) => {
                locations.push(&node.location);
                for container in &node.container_path {
                    locations.push(&container.location);
                }
                for parameter in &node.parameters {
                    locations.push(&parameter.location);
                }
                for clause in &node.clauses {
                    locations.push(&clause.location);
                    for clause_expression in &clause.expressions {
                        match clause_expression {
                            ClauseExpressionNode::Predicate(node) => expression(node, locations),
                            ClauseExpressionNode::Assignment(node) => assignment(node, locations),
                        }
                    }
                }
            }
            DeclarationNode::Scenario(node) => {
                locations.push(&node.location);
                for container in &node.container_path {
                    locations.push(&container.location);
                }
                for item in &node.items {
                    match item {
                        ScenarioItemNode::Given {
                            assignment: node,
                            location,
                        } => {
                            assignment(node, locations);
                            locations.push(location);
                        }
                        ScenarioItemNode::Run {
                            arguments,
                            location,
                            ..
                        } => {
                            for argument in arguments {
                                locations.push(&argument.location);
                            }
                            locations.push(location);
                        }
                        ScenarioItemNode::Expect {
                            expression: node,
                            location,
                        } => {
                            expression(node, locations);
                            locations.push(location);
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Canonical JSON emission
// ---------------------------------------------------------------------------

enum Json {
    Null,
    Bool(bool),
    UInt(u64),
    Str(String),
    Array(Vec<Json>),
    Object(Vec<(&'static str, Json)>),
}

impl Json {
    fn string(value: &str) -> Self {
        Self::Str(value.to_owned())
    }

    fn write(&self, out: &mut String, indent: usize) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(true) => out.push_str("true"),
            Self::Bool(false) => out.push_str("false"),
            Self::UInt(value) => out.push_str(&value.to_string()),
            Self::Str(value) => write_json_string(value, out),
            Self::Array(items) => {
                if items.is_empty() {
                    out.push_str("[]");
                    return;
                }
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    out.push('\n');
                    push_indent(out, indent + 1);
                    item.write(out, indent + 1);
                }
                out.push('\n');
                push_indent(out, indent);
                out.push(']');
            }
            Self::Object(members) => {
                if members.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push('{');
                for (index, (name, value)) in members.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    out.push('\n');
                    push_indent(out, indent + 1);
                    write_json_string(name, out);
                    out.push_str(": ");
                    value.write(out, indent + 1);
                }
                out.push('\n');
                push_indent(out, indent);
                out.push('}');
            }
        }
    }
}

fn push_indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

/// RFC 8259 string escaping with a fixed canonical form: the two mandatory
/// escapes, short escapes for the common control characters, and lowercase
/// `\u00xx` for every other control character. Non-ASCII stays raw UTF-8.
fn write_json_string(value: &str, out: &mut String) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{0c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            character if (character as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => out.push(character),
        }
    }
    out.push('"');
}

fn digest_json(digest: &Digest) -> Json {
    Json::Object(vec![
        ("algorithm", Json::string("sha256")),
        ("value", Json::string(&digest.value)),
    ])
}

fn location_json(location: &ProtocolLocation) -> Json {
    match location {
        ProtocolLocation::Source {
            source_id,
            byte_range,
        } => Json::Object(vec![
            ("kind", Json::string("source")),
            ("source_id", Json::string(source_id)),
            (
                "byte_range",
                Json::Object(vec![
                    ("start", Json::UInt(byte_range.start as u64)),
                    ("end", Json::UInt(byte_range.end as u64)),
                ]),
            ),
        ]),
        ProtocolLocation::Subject => Json::Object(vec![("kind", Json::string("subject"))]),
    }
}

fn details_members(details: &CoverageDetails) -> Vec<(&'static str, Json)> {
    match details {
        CoverageDetails::CompatibilityContainer {
            container_kind,
            name,
        } => vec![
            ("kind", Json::string("compatibility_container")),
            ("container_kind", Json::string(container_kind)),
            ("name", Json::string(name)),
        ],
        CoverageDetails::ActionSoftBehavior { action, behavior } => vec![
            ("kind", Json::string("action_soft_behavior")),
            ("action", Json::string(action)),
            ("behavior", Json::string(behavior)),
        ],
    }
}

fn type_ref_json(type_ref: &TypeRef) -> Json {
    match type_ref {
        TypeRef::Builtin { name } => Json::Object(vec![
            ("kind", Json::string("builtin")),
            ("name", Json::string(name)),
        ]),
        TypeRef::Enum { semantic_key } => Json::Object(vec![
            ("kind", Json::string("enum")),
            ("semantic_key", Json::string(semantic_key)),
        ]),
        TypeRef::Entity { semantic_key } => Json::Object(vec![
            ("kind", Json::string("entity")),
            ("semantic_key", Json::string(semantic_key)),
        ]),
    }
}

fn expression_json(expression: &ExpressionNode) -> Json {
    match expression {
        ExpressionNode::Integer {
            value,
            resolved_type,
            location,
        } => Json::Object(vec![
            ("kind", Json::string("integer")),
            ("value", Json::string(value)),
            ("resolved_type", type_ref_json(resolved_type)),
            ("location", location_json(location)),
        ]),
        ExpressionNode::Boolean {
            value,
            resolved_type,
            location,
        } => Json::Object(vec![
            ("kind", Json::string("boolean")),
            ("value", Json::Bool(*value)),
            ("resolved_type", type_ref_json(resolved_type)),
            ("location", location_json(location)),
        ]),
        ExpressionNode::EnumMember {
            enum_semantic_key,
            member_semantic_key,
            member,
            resolved_type,
            location,
        } => Json::Object(vec![
            ("kind", Json::string("enum_member")),
            ("enum_semantic_key", Json::string(enum_semantic_key)),
            ("member_semantic_key", Json::string(member_semantic_key)),
            ("member", Json::string(member)),
            ("resolved_type", type_ref_json(resolved_type)),
            ("location", location_json(location)),
        ]),
        ExpressionNode::Path(path) => path_expression_json(path),
        ExpressionNode::Binary {
            operator,
            left,
            right,
            resolved_type,
            location,
        } => Json::Object(vec![
            ("kind", Json::string("binary")),
            ("operator", Json::string(operator)),
            ("left", expression_json(left)),
            ("right", expression_json(right)),
            ("resolved_type", type_ref_json(resolved_type)),
            ("location", location_json(location)),
        ]),
        ExpressionNode::Not {
            operand,
            resolved_type,
            location,
        } => Json::Object(vec![
            ("kind", Json::string("not")),
            ("operand", expression_json(operand)),
            ("resolved_type", type_ref_json(resolved_type)),
            ("location", location_json(location)),
        ]),
        ExpressionNode::Or {
            left,
            right,
            resolved_type,
            location,
        } => Json::Object(vec![
            ("kind", Json::string("or")),
            ("left", expression_json(left)),
            ("right", expression_json(right)),
            ("resolved_type", type_ref_json(resolved_type)),
            ("location", location_json(location)),
        ]),
    }
}

fn path_expression_json(path: &PathExpressionNode) -> Json {
    let root = match &path.root {
        PathRootNode::ActionParameter { semantic_key } => Json::Object(vec![
            ("kind", Json::string("action_parameter")),
            ("semantic_key", Json::string(semantic_key)),
        ]),
        PathRootNode::EntitySelf { semantic_key } => Json::Object(vec![
            ("kind", Json::string("entity_self")),
            ("semantic_key", Json::string(semantic_key)),
        ]),
        PathRootNode::ScenarioInstance {
            name,
            instance_type,
        } => Json::Object(vec![
            ("kind", Json::string("scenario_instance")),
            ("name", Json::string(name)),
            ("type", type_ref_json(instance_type)),
        ]),
    };
    Json::Object(vec![
        ("kind", Json::string("path")),
        ("root", root),
        (
            "segments",
            Json::Array(
                path.segments
                    .iter()
                    .map(|segment| {
                        Json::Object(vec![
                            ("name", Json::string(&segment.name)),
                            ("resolved_type", type_ref_json(&segment.resolved_type)),
                            ("location", location_json(&segment.location)),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("resolved_type", type_ref_json(&path.resolved_type)),
        ("location", location_json(&path.location)),
    ])
}

fn assignment_json(assignment: &AssignmentNode) -> Json {
    Json::Object(vec![
        ("target", path_expression_json(&assignment.target)),
        ("operator", Json::string(assignment.operator)),
        ("value", expression_json(&assignment.value)),
        ("target_type", type_ref_json(&assignment.target_type)),
        ("location", location_json(&assignment.location)),
    ])
}

fn container_path_json(container_path: &[ContainerRef]) -> Json {
    Json::Array(
        container_path
            .iter()
            .map(|container| {
                Json::Object(vec![
                    ("container_kind", Json::string(&container.container_kind)),
                    ("name", Json::string(&container.name)),
                    ("location", location_json(&container.location)),
                ])
            })
            .collect(),
    )
}

fn declaration_json(declaration: &DeclarationNode) -> Json {
    match declaration {
        DeclarationNode::Enum(node) => Json::Object(vec![
            ("kind", Json::string("enum")),
            ("node_id", Json::string(&node.node_id)),
            ("semantic_key", Json::string(&node.semantic_key)),
            ("name", Json::string(&node.name)),
            ("location", location_json(&node.location)),
            ("container_path", container_path_json(&node.container_path)),
            (
                "members",
                Json::Array(
                    node.members
                        .iter()
                        .map(|member| {
                            Json::Object(vec![
                                ("kind", Json::string("enum_member")),
                                ("node_id", Json::string(&member.node_id)),
                                ("semantic_key", Json::string(&member.semantic_key)),
                                ("name", Json::string(&member.name)),
                                ("location", location_json(&member.location)),
                                ("enum_semantic_key", Json::string(&member.enum_semantic_key)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]),
        DeclarationNode::Entity(node) => Json::Object(vec![
            ("kind", Json::string("entity")),
            ("node_id", Json::string(&node.node_id)),
            ("semantic_key", Json::string(&node.semantic_key)),
            ("name", Json::string(&node.name)),
            ("location", location_json(&node.location)),
            ("container_path", container_path_json(&node.container_path)),
            (
                "fields",
                Json::Array(
                    node.fields
                        .iter()
                        .map(|field| {
                            Json::Object(vec![
                                ("kind", Json::string("field")),
                                ("node_id", Json::string(&field.node_id)),
                                ("semantic_key", Json::string(&field.semantic_key)),
                                ("name", Json::string(&field.name)),
                                ("location", location_json(&field.location)),
                                ("type", type_ref_json(&field.field_type)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "invariants",
                Json::Array(node.invariants.iter().map(expression_json).collect()),
            ),
        ]),
        DeclarationNode::Action(node) => Json::Object(vec![
            ("kind", Json::string("action")),
            ("node_id", Json::string(&node.node_id)),
            ("semantic_key", Json::string(&node.semantic_key)),
            ("name", Json::string(&node.name)),
            ("location", location_json(&node.location)),
            ("container_path", container_path_json(&node.container_path)),
            (
                "parameters",
                Json::Array(
                    node.parameters
                        .iter()
                        .map(|parameter| {
                            Json::Object(vec![
                                ("kind", Json::string("parameter")),
                                ("node_id", Json::string(&parameter.node_id)),
                                ("semantic_key", Json::string(&parameter.semantic_key)),
                                ("name", Json::string(&parameter.name)),
                                ("location", location_json(&parameter.location)),
                                (
                                    "action_semantic_key",
                                    Json::string(&parameter.action_semantic_key),
                                ),
                                ("type", type_ref_json(&parameter.parameter_type)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "clauses",
                Json::Array(
                    node.clauses
                        .iter()
                        .map(|clause| {
                            Json::Object(vec![
                                ("kind", Json::string("clause")),
                                ("clause_kind", Json::string(clause.clause_kind)),
                                ("state_phase", Json::string(clause.state_phase)),
                                (
                                    "expressions",
                                    Json::Array(
                                        clause
                                            .expressions
                                            .iter()
                                            .map(|expression| match expression {
                                                ClauseExpressionNode::Predicate(node) => {
                                                    Json::Object(vec![
                                                        ("kind", Json::string("predicate")),
                                                        ("expression", expression_json(node)),
                                                    ])
                                                }
                                                ClauseExpressionNode::Assignment(node) => {
                                                    Json::Object(vec![
                                                        ("kind", Json::string("assignment")),
                                                        ("assignment", assignment_json(node)),
                                                    ])
                                                }
                                            })
                                            .collect(),
                                    ),
                                ),
                                ("location", location_json(&clause.location)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]),
        DeclarationNode::Scenario(node) => Json::Object(vec![
            ("kind", Json::string("scenario")),
            ("node_id", Json::string(&node.node_id)),
            ("semantic_key", Json::string(&node.semantic_key)),
            ("name", Json::string(&node.name)),
            ("location", location_json(&node.location)),
            ("container_path", container_path_json(&node.container_path)),
            (
                "items",
                Json::Array(
                    node.items
                        .iter()
                        .map(|item| match item {
                            ScenarioItemNode::Given {
                                assignment,
                                location,
                            } => Json::Object(vec![
                                ("kind", Json::string("given")),
                                ("assignment", assignment_json(assignment)),
                                ("location", location_json(location)),
                            ]),
                            ScenarioItemNode::Run {
                                action_semantic_key,
                                arguments,
                                location,
                            } => Json::Object(vec![
                                ("kind", Json::string("run")),
                                ("action_semantic_key", Json::string(action_semantic_key)),
                                (
                                    "arguments",
                                    Json::Array(
                                        arguments
                                            .iter()
                                            .map(|argument| {
                                                Json::Object(vec![
                                                    ("name", Json::string(&argument.name)),
                                                    (
                                                        "parameter_semantic_key",
                                                        Json::string(
                                                            &argument.parameter_semantic_key,
                                                        ),
                                                    ),
                                                    (
                                                        "type",
                                                        type_ref_json(&argument.argument_type),
                                                    ),
                                                    ("location", location_json(&argument.location)),
                                                ])
                                            })
                                            .collect(),
                                    ),
                                ),
                                ("location", location_json(location)),
                            ]),
                            ScenarioItemNode::Expect {
                                expression,
                                location,
                            } => Json::Object(vec![
                                ("kind", Json::string("expect")),
                                ("expression", expression_json(expression)),
                                ("location", location_json(location)),
                            ]),
                        })
                        .collect(),
                ),
            ),
        ]),
    }
}

impl CheckedSemanticsDocument {
    pub fn to_canonical_json(&self) -> Result<String, ProtocolSerializationError> {
        self.validate().map_err(ProtocolSerializationError)?;
        let result = &self.result;
        let findings = Json::Array(
            result
                .findings
                .iter()
                .map(|finding| {
                    let mut members = vec![
                        ("finding_id", Json::string(&finding.finding_id)),
                        (
                            "severity",
                            Json::string(match finding.severity {
                                Severity::Error => "error",
                                Severity::Warning => "warning",
                            }),
                        ),
                        (
                            "category",
                            Json::string(match finding.category {
                                FindingCategory::Lexical => "lexical",
                                FindingCategory::Syntax => "syntax",
                                FindingCategory::Project => "project",
                                FindingCategory::Semantic => "semantic",
                                FindingCategory::Coverage => "coverage",
                            }),
                        ),
                        ("code", Json::string(&finding.code)),
                        ("message", Json::string(&finding.message)),
                        ("primary_location", location_json(&finding.primary_location)),
                    ];
                    if let Some(details) = &finding.details {
                        members.push(("details", Json::Object(details_members(details))));
                    }
                    Json::Object(members)
                })
                .collect(),
        );
        let coverage = Json::Object(vec![
            (
                "assessment",
                Json::string(match result.coverage.assessment {
                    CoverageAssessment::Complete => "complete",
                    CoverageAssessment::Unavailable => "unavailable",
                }),
            ),
            (
                "fully_modeled",
                match result.coverage.fully_modeled {
                    Some(value) => Json::Bool(value),
                    None => Json::Null,
                },
            ),
            (
                "unmodeled",
                Json::Array(
                    result
                        .coverage
                        .unmodeled
                        .iter()
                        .map(|item| {
                            let mut members = details_members(&item.details);
                            members.push(("location", location_json(&item.location)));
                            Json::Object(members)
                        })
                        .collect(),
                ),
            ),
        ]);
        let checked_model = match &result.checked_model {
            None => Json::Null,
            Some(model) => Json::Object(vec![
                ("language", Json::string(PROTOCOL_LANGUAGE)),
                ("language_version", Json::string(PROTOCOL_LANGUAGE_VERSION)),
                (
                    "system",
                    Json::Object(vec![
                        ("kind", Json::string("system")),
                        ("node_id", Json::string(&model.system.node_id)),
                        ("semantic_key", Json::string(&model.system.semantic_key)),
                        ("name", Json::string(&model.system.name)),
                        (
                            "shell_locations",
                            Json::Array(
                                model
                                    .system
                                    .shell_locations
                                    .iter()
                                    .map(location_json)
                                    .collect(),
                            ),
                        ),
                        (
                            "declarations",
                            Json::Array(
                                model
                                    .system
                                    .declarations
                                    .iter()
                                    .map(declaration_json)
                                    .collect(),
                            ),
                        ),
                    ]),
                ),
            ]),
        };
        let document = Json::Object(vec![
            ("protocol", Json::string(PROTOCOL_NAME)),
            ("version", Json::UInt(PROTOCOL_VERSION)),
            (
                "capabilities",
                Json::Array(
                    PROTOCOL_CAPABILITIES
                        .iter()
                        .map(|capability| Json::string(capability))
                        .collect(),
                ),
            ),
            (
                "producer",
                Json::Object(vec![
                    ("name", Json::string("morva")),
                    ("version", Json::string(&self.producer_version)),
                ]),
            ),
            (
                "subject",
                Json::Object(vec![
                    (
                        "kind",
                        Json::string(match self.subject.kind {
                            SubjectKind::File => "file",
                        }),
                    ),
                    ("name", Json::string(&self.subject.name)),
                    ("revision", digest_json(&self.subject.revision)),
                ]),
            ),
            (
                "sources",
                Json::Array(
                    self.sources
                        .iter()
                        .map(|source| {
                            Json::Object(vec![
                                ("source_id", Json::string(&source.source_id)),
                                ("name", Json::string(&source.name)),
                                ("content_encoding", Json::string("utf-8")),
                                ("content", Json::string(&source.content)),
                                ("revision", digest_json(&source.revision)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "result",
                Json::Object(vec![
                    (
                        "status",
                        Json::string(match result.status {
                            ResultStatus::Valid => "valid",
                            ResultStatus::Invalid => "invalid",
                        }),
                    ),
                    ("findings", findings),
                    ("coverage", coverage),
                    ("checked_model", checked_model),
                ]),
            ),
        ]);
        let mut out = String::new();
        document.write(&mut out, 0);
        out.push('\n');
        Ok(out)
    }
}
