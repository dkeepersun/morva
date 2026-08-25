use std::collections::{BTreeMap, HashMap};
use std::fmt;

use crate::Diagnostic;
use crate::ast::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Boolean(bool),
    Integer(i64),
    Enum { type_name: String, member: String },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Integer(value) => write!(f, "{value}"),
            Self::Enum { member, .. } => f.write_str(member),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationPhase {
    Givens,
    InitialInvariants,
    Requires,
    Effects,
    FinalInvariants,
    Ensures,
    Expects,
}

impl SimulationPhase {
    pub const ALL: [Self; 7] = [
        Self::Givens,
        Self::InitialInvariants,
        Self::Requires,
        Self::Effects,
        Self::FinalInvariants,
        Self::Ensures,
        Self::Expects,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Givens => "givens",
            Self::InitialInvariants => "initial invariants",
            Self::Requires => "requires",
            Self::Effects => "effects",
            Self::FinalInvariants => "final invariants",
            Self::Ensures => "ensures",
            Self::Expects => "expects",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseResult {
    pub phase: SimulationPhase,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateChange {
    pub path: String,
    pub before: Option<Value>,
    pub after: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationFailure {
    pub phase: SimulationPhase,
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationReport {
    pub scenario: String,
    pub action: String,
    pub phases: Vec<PhaseResult>,
    pub changes: Vec<StateChange>,
    pub state: BTreeMap<String, Value>,
    pub failure: Option<SimulationFailure>,
}

impl SimulationReport {
    pub fn succeeded(&self) -> bool {
        self.failure.is_none()
    }
}

struct Model<'a> {
    actions: HashMap<&'a str, Vec<&'a Action>>,
    scenarios: HashMap<&'a str, Vec<&'a Scenario>>,
    entities: HashMap<&'a str, Vec<&'a Entity>>,
    enums: HashMap<&'a str, Vec<&'a Enum>>,
}

struct Binding<'a> {
    parameter: &'a Parameter,
    argument: &'a Name,
    entity: &'a Entity,
}

#[derive(Default)]
struct EvaluationContext<'a> {
    aliases: HashMap<&'a str, &'a str>,
    entity_argument: Option<&'a str>,
    entity: Option<&'a Entity>,
}

struct RuntimeError {
    message: String,
    span: Span,
}

pub fn simulate(document: &Document, scenario_name: &str) -> Result<SimulationReport, Diagnostic> {
    if let Some(diagnostic) = crate::check(document).into_iter().next() {
        return Err(diagnostic);
    }
    let mut model = Model {
        actions: HashMap::new(),
        scenarios: HashMap::new(),
        entities: HashMap::new(),
        enums: HashMap::new(),
    };
    collect_model(&document.declarations, &mut model);
    let scenarios = model.scenarios.get(scenario_name).ok_or_else(|| {
        Diagnostic::new(
            "MORVA4001",
            format!("unknown scenario '{scenario_name}'"),
            document.span,
        )
    })?;
    if scenarios.len() != 1 {
        return Err(Diagnostic::new(
            "MORVA4001",
            format!("scenario '{scenario_name}' is ambiguous"),
            scenarios[1].name.span,
        ));
    }
    let scenario = scenarios[0];
    let run = scenario
        .items
        .iter()
        .find_map(|item| match item {
            ScenarioItem::Run(run) => Some(run),
            _ => None,
        })
        .ok_or_else(|| {
            Diagnostic::new(
                "MORVA4002",
                "scenario has no runnable action",
                scenario.name.span,
            )
        })?;
    let actions = model.actions.get(run.action.text.as_str()).ok_or_else(|| {
        Diagnostic::new(
            "MORVA4002",
            format!("unknown action '{}'", run.action.text),
            run.action.span,
        )
    })?;
    if actions.len() != 1 {
        return Err(Diagnostic::new(
            "MORVA4002",
            format!("action '{}' is ambiguous", run.action.text),
            run.action.span,
        ));
    }
    let action = actions[0];
    let bindings = build_bindings(action, run, &model).ok_or_else(|| {
        Diagnostic::new(
            "MORVA4003",
            "scenario action binding is not simulatable",
            run.span,
        )
    })?;
    let aliases = bindings
        .iter()
        .map(|binding| {
            (
                binding.parameter.name.text.as_str(),
                binding.argument.text.as_str(),
            )
        })
        .collect();
    let action_context = EvaluationContext {
        aliases,
        entity_argument: None,
        entity: None,
    };
    let scenario_context = EvaluationContext::default();
    let mut report = SimulationReport {
        scenario: scenario.name.text.clone(),
        action: action.name.text.clone(),
        phases: Vec::new(),
        changes: Vec::new(),
        state: BTreeMap::new(),
        failure: None,
    };

    for item in &scenario.items {
        if let ScenarioItem::Given(assignment) = item {
            if assignment.operator != AssignmentOperator::Set {
                return Ok(fail(
                    report,
                    SimulationPhase::Givens,
                    "given only supports '=' initialization",
                    assignment.span,
                ));
            }
            let Some(field) = scenario_field(&assignment.target, &bindings) else {
                return Ok(fail(
                    report,
                    SimulationPhase::Givens,
                    "given target is not a bound entity field",
                    assignment.target.span,
                ));
            };
            let value = match evaluate_value(
                &assignment.value,
                Some(&field.type_name),
                &scenario_context,
                &report.state,
                &model,
            ) {
                Ok(value) => value,
                Err(error) => {
                    return Ok(fail(
                        report,
                        SimulationPhase::Givens,
                        error.message,
                        error.span,
                    ));
                }
            };
            let path = assignment.target.display();
            if report.state.contains_key(&path) {
                return Ok(fail(
                    report,
                    SimulationPhase::Givens,
                    format!("duplicate initialization of '{path}'"),
                    assignment.target.span,
                ));
            }
            report.state.insert(path, value);
        }
    }
    pass(&mut report, SimulationPhase::Givens);

    for binding in &bindings {
        let context = EvaluationContext {
            aliases: HashMap::new(),
            entity_argument: Some(&binding.argument.text),
            entity: Some(binding.entity),
        };
        for invariant in &binding.entity.invariants {
            if let Err(error) = require_true(invariant, &context, &report.state, &model) {
                return Ok(fail(
                    report,
                    SimulationPhase::InitialInvariants,
                    error.message,
                    error.span,
                ));
            }
        }
    }
    pass(&mut report, SimulationPhase::InitialInvariants);

    for expression in action_predicates(action, &[ClauseKind::Requires, ClauseKind::Invariant]) {
        if let Err(error) = require_true(expression, &action_context, &report.state, &model) {
            return Ok(fail(
                report,
                SimulationPhase::Requires,
                error.message,
                error.span,
            ));
        }
    }
    pass(&mut report, SimulationPhase::Requires);

    for assignment in action_effects(action) {
        let Some((path, field)) = action_field(&assignment.target, &bindings) else {
            return Ok(fail(
                report,
                SimulationPhase::Effects,
                "effect target is not a directly bound entity field",
                assignment.target.span,
            ));
        };
        let value = match evaluate_value(
            &assignment.value,
            Some(&field.type_name),
            &action_context,
            &report.state,
            &model,
        ) {
            Ok(value) => value,
            Err(error) => {
                return Ok(fail(
                    report,
                    SimulationPhase::Effects,
                    error.message,
                    error.span,
                ));
            }
        };
        if let Err(error) =
            ensure_value_type(&value, &field.type_name, &model, assignment.value.span)
        {
            return Ok(fail(
                report,
                SimulationPhase::Effects,
                error.message,
                error.span,
            ));
        }
        let before = report.state.get(&path).cloned();
        let after = match assignment.operator {
            AssignmentOperator::Set => value,
            AssignmentOperator::Add | AssignmentOperator::Subtract => {
                let Some(Value::Integer(current)) = before.as_ref() else {
                    return Ok(fail(
                        report,
                        SimulationPhase::Effects,
                        format!("uninitialized or non-Integer compound target '{path}'"),
                        assignment.target.span,
                    ));
                };
                let Value::Integer(delta) = value else {
                    return Ok(fail(
                        report,
                        SimulationPhase::Effects,
                        "compound assignment value must be Integer",
                        assignment.value.span,
                    ));
                };
                let result = match assignment.operator {
                    AssignmentOperator::Add => current.checked_add(delta),
                    AssignmentOperator::Subtract => current.checked_sub(delta),
                    AssignmentOperator::Set => unreachable!(),
                };
                let Some(result) = result else {
                    return Ok(fail(
                        report,
                        SimulationPhase::Effects,
                        "Integer arithmetic overflow",
                        assignment.span,
                    ));
                };
                Value::Integer(result)
            }
        };
        report.state.insert(path.clone(), after.clone());
        report.changes.push(StateChange {
            path,
            before,
            after,
        });
    }
    pass(&mut report, SimulationPhase::Effects);

    for expression in action_predicates(action, &[ClauseKind::Invariant]) {
        if let Err(error) = require_true(expression, &action_context, &report.state, &model) {
            return Ok(fail(
                report,
                SimulationPhase::FinalInvariants,
                error.message,
                error.span,
            ));
        }
    }
    for binding in &bindings {
        let context = EvaluationContext {
            aliases: HashMap::new(),
            entity_argument: Some(&binding.argument.text),
            entity: Some(binding.entity),
        };
        for invariant in &binding.entity.invariants {
            if let Err(error) = require_true(invariant, &context, &report.state, &model) {
                return Ok(fail(
                    report,
                    SimulationPhase::FinalInvariants,
                    error.message,
                    error.span,
                ));
            }
        }
    }
    pass(&mut report, SimulationPhase::FinalInvariants);

    for expression in action_predicates(action, &[ClauseKind::Ensures]) {
        if let Err(error) = require_true(expression, &action_context, &report.state, &model) {
            return Ok(fail(
                report,
                SimulationPhase::Ensures,
                error.message,
                error.span,
            ));
        }
    }
    pass(&mut report, SimulationPhase::Ensures);

    for item in &scenario.items {
        if let ScenarioItem::Expect(expression) = item
            && let Err(error) = require_true(expression, &scenario_context, &report.state, &model)
        {
            return Ok(fail(
                report,
                SimulationPhase::Expects,
                error.message,
                error.span,
            ));
        }
    }
    pass(&mut report, SimulationPhase::Expects);
    Ok(report)
}

fn collect_model<'a>(declarations: &'a [Declaration], model: &mut Model<'a>) {
    for declaration in declarations {
        match declaration {
            Declaration::Action(action) => model
                .actions
                .entry(&action.name.text)
                .or_default()
                .push(action),
            Declaration::Scenario(scenario) => model
                .scenarios
                .entry(&scenario.name.text)
                .or_default()
                .push(scenario),
            Declaration::Entity(entity) => model
                .entities
                .entry(&entity.name.text)
                .or_default()
                .push(entity),
            Declaration::Enum(enumeration) => model
                .enums
                .entry(&enumeration.name.text)
                .or_default()
                .push(enumeration),
            _ => {}
        }
        collect_model(declaration.declarations(), model);
    }
}

fn build_bindings<'a>(
    action: &'a Action,
    run: &'a Run,
    model: &Model<'a>,
) -> Option<Vec<Binding<'a>>> {
    if action.parameters.len() != run.arguments.len() {
        return None;
    }
    action
        .parameters
        .iter()
        .zip(&run.arguments)
        .map(|(parameter, argument)| {
            let entities = model.entities.get(parameter.type_name.text.as_str())?;
            (entities.len() == 1).then(|| Binding {
                parameter,
                argument,
                entity: entities[0],
            })
        })
        .collect()
}

fn scenario_field<'a>(path: &Path, bindings: &'a [Binding<'a>]) -> Option<&'a Field> {
    if path.segments.len() != 2 {
        return None;
    }
    let binding = bindings
        .iter()
        .find(|binding| binding.argument.text == path.segments[0].text)?;
    let field = binding
        .entity
        .fields
        .iter()
        .find(|field| field.name.text == path.segments[1].text)?;
    Some(field)
}

fn action_field<'a>(path: &Path, bindings: &'a [Binding<'a>]) -> Option<(String, &'a Field)> {
    if path.segments.len() != 2 {
        return None;
    }
    let binding = bindings
        .iter()
        .find(|binding| binding.parameter.name.text == path.segments[0].text)?;
    let field = binding
        .entity
        .fields
        .iter()
        .find(|field| field.name.text == path.segments[1].text)?;
    Some((
        format!("{}.{}", binding.argument.text, field.name.text),
        field,
    ))
}

fn action_predicates<'a>(action: &'a Action, kinds: &[ClauseKind]) -> Vec<&'a Expr> {
    action
        .clauses
        .iter()
        .filter(|clause| kinds.contains(&clause.kind))
        .flat_map(|clause| &clause.expressions)
        .filter_map(|expression| match expression {
            ClauseExpression::Predicate(expression) => Some(expression),
            ClauseExpression::Assignment(_) => None,
        })
        .collect()
}

fn action_effects(action: &Action) -> Vec<&Assignment> {
    action
        .clauses
        .iter()
        .filter(|clause| clause.kind == ClauseKind::Effects)
        .flat_map(|clause| &clause.expressions)
        .filter_map(|expression| match expression {
            ClauseExpression::Assignment(assignment) => Some(assignment),
            ClauseExpression::Predicate(_) => None,
        })
        .collect()
}

fn require_true(
    expression: &Expr,
    context: &EvaluationContext<'_>,
    state: &BTreeMap<String, Value>,
    model: &Model<'_>,
) -> Result<(), RuntimeError> {
    match evaluate_value(expression, None, context, state, model)? {
        Value::Boolean(true) => Ok(()),
        Value::Boolean(false) => Err(RuntimeError {
            message: "predicate evaluated to false".to_owned(),
            span: expression.span,
        }),
        _ => Err(RuntimeError {
            message: "predicate did not evaluate to Boolean".to_owned(),
            span: expression.span,
        }),
    }
}

fn evaluate_value(
    expression: &Expr,
    expected: Option<&Name>,
    context: &EvaluationContext<'_>,
    state: &BTreeMap<String, Value>,
    model: &Model<'_>,
) -> Result<Value, RuntimeError> {
    match &expression.kind {
        ExprKind::Integer(value) => {
            ensure_literal_type(expected, &["Int", "Integer"], expression.span)?;
            Ok(Value::Integer(*value))
        }
        ExprKind::Boolean(value) => {
            ensure_literal_type(expected, &["Bool", "Boolean"], expression.span)?;
            Ok(Value::Boolean(*value))
        }
        ExprKind::Path(path) => evaluate_path(path, expected, context, state, model),
        ExprKind::Binary {
            left,
            operator,
            right,
        } => evaluate_binary(
            left,
            *operator,
            right,
            expression.span,
            context,
            state,
            model,
        ),
    }
}

fn evaluate_binary(
    left: &Expr,
    operator: BinaryOperator,
    right: &Expr,
    span: Span,
    context: &EvaluationContext<'_>,
    state: &BTreeMap<String, Value>,
    model: &Model<'_>,
) -> Result<Value, RuntimeError> {
    let left_bare = bare_unbound(left, context);
    let right_bare = bare_unbound(right, context);
    let (left, right) = if let Some(name) = left_bare {
        let right = evaluate_value(right, None, context, state, model)?;
        let left = evaluate_enum_name(name, &right, model)?;
        (left, right)
    } else if let Some(name) = right_bare {
        let left = evaluate_value(left, None, context, state, model)?;
        let right = evaluate_enum_name(name, &left, model)?;
        (left, right)
    } else {
        (
            evaluate_value(left, None, context, state, model)?,
            evaluate_value(right, None, context, state, model)?,
        )
    };
    let result = match operator {
        BinaryOperator::Equal | BinaryOperator::NotEqual => {
            if !same_value_type(&left, &right) {
                return Err(RuntimeError {
                    message: "equality comparison requires values of the same type".to_owned(),
                    span,
                });
            }
            if operator == BinaryOperator::Equal {
                left == right
            } else {
                left != right
            }
        }
        BinaryOperator::Greater
        | BinaryOperator::GreaterEqual
        | BinaryOperator::Less
        | BinaryOperator::LessEqual => {
            let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
                return Err(RuntimeError {
                    message: "ordered comparison requires Integer values".to_owned(),
                    span,
                });
            };
            match operator {
                BinaryOperator::Greater => left > right,
                BinaryOperator::GreaterEqual => left >= right,
                BinaryOperator::Less => left < right,
                BinaryOperator::LessEqual => left <= right,
                BinaryOperator::Equal | BinaryOperator::NotEqual => unreachable!(),
            }
        }
    };
    Ok(Value::Boolean(result))
}

fn ensure_literal_type(
    expected: Option<&Name>,
    supported: &[&str],
    span: Span,
) -> Result<(), RuntimeError> {
    if let Some(expected) = expected
        && !supported.contains(&expected.text.as_str())
    {
        return Err(RuntimeError {
            message: format!("unsupported value type '{}'", expected.text),
            span,
        });
    }
    Ok(())
}

fn bare_unbound<'a>(expression: &'a Expr, context: &EvaluationContext<'_>) -> Option<&'a Name> {
    let ExprKind::Path(path) = &expression.kind else {
        return None;
    };
    let name = path.segments.first()?;
    let is_entity_field = context.entity.is_some_and(|entity| {
        entity
            .fields
            .iter()
            .any(|field| field.name.text == name.text)
    });
    (path.segments.len() == 1
        && !is_entity_field
        && !context.aliases.contains_key(name.text.as_str()))
    .then_some(name)
}

fn evaluate_path(
    path: &Path,
    expected: Option<&Name>,
    context: &EvaluationContext<'_>,
    state: &BTreeMap<String, Value>,
    model: &Model<'_>,
) -> Result<Value, RuntimeError> {
    if let Some(key) = state_key(path, context) {
        return state.get(&key).cloned().ok_or_else(|| RuntimeError {
            message: format!("uninitialized read of '{key}'"),
            span: path.span,
        });
    }
    if path.segments.len() == 1
        && let Some(type_name) = expected
    {
        return enum_member_value(&path.segments[0], type_name, model);
    }
    Err(RuntimeError {
        message: format!("uninitialized read of '{}'", path.display()),
        span: path.span,
    })
}

fn state_key(path: &Path, context: &EvaluationContext<'_>) -> Option<String> {
    let root = path.segments.first()?;
    if let Some(argument) = context.aliases.get(root.text.as_str()) {
        return Some(
            std::iter::once(*argument)
                .chain(path.segments[1..].iter().map(|item| item.text.as_str()))
                .collect::<Vec<_>>()
                .join("."),
        );
    }
    if let Some(argument) = context.entity_argument
        && path.segments.len() == 1
    {
        return Some(format!("{argument}.{}", path.segments[0].text));
    }
    (path.segments.len() > 1).then(|| path.display())
}

fn same_value_type(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Boolean(_), Value::Boolean(_)) | (Value::Integer(_), Value::Integer(_)) => true,
        (
            Value::Enum {
                type_name: left, ..
            },
            Value::Enum {
                type_name: right, ..
            },
        ) => left == right,
        _ => false,
    }
}

fn ensure_value_type(
    value: &Value,
    expected: &Name,
    model: &Model<'_>,
    span: Span,
) -> Result<(), RuntimeError> {
    let matches = match value {
        Value::Boolean(_) => matches!(expected.text.as_str(), "Bool" | "Boolean"),
        Value::Integer(_) => matches!(expected.text.as_str(), "Int" | "Integer"),
        Value::Enum { type_name, .. } => {
            type_name == &expected.text && model.enums.contains_key(expected.text.as_str())
        }
    };
    if matches {
        Ok(())
    } else {
        Err(RuntimeError {
            message: format!("value is not compatible with type '{}'", expected.text),
            span,
        })
    }
}

fn evaluate_enum_name(
    name: &Name,
    other: &Value,
    model: &Model<'_>,
) -> Result<Value, RuntimeError> {
    let Value::Enum { type_name, .. } = other else {
        return Err(RuntimeError {
            message: format!("uninitialized read of '{}'", name.text),
            span: name.span,
        });
    };
    enum_member_value(
        name,
        &Name {
            text: type_name.clone(),
            span: name.span,
        },
        model,
    )
}

fn enum_member_value(
    member: &Name,
    type_name: &Name,
    model: &Model<'_>,
) -> Result<Value, RuntimeError> {
    let Some(enumerations) = model.enums.get(type_name.text.as_str()) else {
        return Err(RuntimeError {
            message: format!("unsupported value type '{}'", type_name.text),
            span: member.span,
        });
    };
    if enumerations.len() != 1
        || !enumerations[0]
            .members
            .iter()
            .any(|item| item.text == member.text)
    {
        return Err(RuntimeError {
            message: format!(
                "unknown member '{}' for enum '{}'",
                member.text, type_name.text
            ),
            span: member.span,
        });
    }
    Ok(Value::Enum {
        type_name: type_name.text.clone(),
        member: member.text.clone(),
    })
}

fn pass(report: &mut SimulationReport, phase: SimulationPhase) {
    report.phases.push(PhaseResult {
        phase,
        passed: true,
    });
}

fn fail(
    mut report: SimulationReport,
    phase: SimulationPhase,
    message: impl Into<String>,
    span: Span,
) -> SimulationReport {
    report.phases.push(PhaseResult {
        phase,
        passed: false,
    });
    report.failure = Some(SimulationFailure {
        phase,
        message: message.into(),
        span,
    });
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_equality_evaluator_rejects_different_value_types() {
        let left = Expr {
            kind: ExprKind::Integer(1),
            span: Span { start: 4, end: 5 },
        };
        let right = Expr {
            kind: ExprKind::Boolean(true),
            span: Span { start: 9, end: 13 },
        };
        let comparison_span = Span { start: 4, end: 13 };
        let context = EvaluationContext::default();
        let state = BTreeMap::new();
        let model = Model {
            actions: HashMap::new(),
            scenarios: HashMap::new(),
            entities: HashMap::new(),
            enums: HashMap::new(),
        };

        let error = evaluate_binary(
            &left,
            BinaryOperator::Equal,
            &right,
            comparison_span,
            &context,
            &state,
            &model,
        )
        .expect_err("runtime equality guard must reject different value types");

        assert_eq!(
            error.message,
            "equality comparison requires values of the same type"
        );
        assert_eq!(error.span, comparison_span);
    }

    #[test]
    fn runtime_effect_guard_preserves_expected_field_type() {
        let model = Model {
            actions: HashMap::new(),
            scenarios: HashMap::new(),
            entities: HashMap::new(),
            enums: HashMap::new(),
        };
        let integer = Name {
            text: "Integer".to_owned(),
            span: Span::default(),
        };
        let int_alias = Name {
            text: "Int".to_owned(),
            span: Span::default(),
        };
        assert!(ensure_value_type(&Value::Integer(1), &int_alias, &model, Span::default()).is_ok());
        let error = ensure_value_type(&Value::Boolean(true), &integer, &model, Span::default())
            .expect_err("runtime guard must reject a mismatched effect value");
        assert_eq!(error.message, "value is not compatible with type 'Integer'");
    }
}
