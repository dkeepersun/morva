use std::collections::{HashMap, HashSet};

use crate::Diagnostic;
use crate::ast::*;

#[derive(Clone, Copy)]
enum TypeDeclaration<'a> {
    Entity(&'a Entity),
    Enum(&'a Enum),
}

impl TypeDeclaration<'_> {
    fn name(&self) -> &Name {
        match self {
            Self::Entity(item) => &item.name,
            Self::Enum(item) => &item.name,
        }
    }
}

struct TypeIndex<'a> {
    declarations: HashMap<&'a str, Vec<TypeDeclaration<'a>>>,
}

struct ExecutableIndex<'a> {
    actions: HashMap<&'a str, Vec<&'a Action>>,
    scenarios: HashMap<&'a str, Vec<&'a Scenario>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BuiltinType {
    Boolean,
    Decimal,
    Id,
    Integer,
    String,
}

impl BuiltinType {
    fn display(self) -> &'static str {
        match self {
            Self::Boolean => "Boolean",
            Self::Decimal => "Decimal",
            Self::Id => "ID",
            Self::Integer => "Integer",
            Self::String => "String",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ResolvedType<'a> {
    Builtin(BuiltinType),
    Entity(&'a Entity),
    Enum(&'a Enum),
}

enum Operand<'a, 'b> {
    Typed(Option<ResolvedType<'a>>),
    Unbound(&'b Name),
}

pub(crate) fn check(document: &Document) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let top_level_systems = document
        .declarations
        .iter()
        .filter(|item| matches!(item, Declaration::System(_)))
        .count();
    if top_level_systems != 1 {
        diagnostics.push(Diagnostic::new(
            "MORVA2001",
            format!("expected exactly one top-level system declaration, found {top_level_systems}"),
            document.span,
        ));
    }

    find_nested_systems(&document.declarations, true, &mut diagnostics);
    let mut index = TypeIndex {
        declarations: HashMap::new(),
    };
    collect_types(&document.declarations, &mut index);
    check_global_type_ambiguities(&index, &mut diagnostics);
    let mut executables = ExecutableIndex {
        actions: HashMap::new(),
        scenarios: HashMap::new(),
    };
    collect_executables(&document.declarations, &mut executables);
    check_global_executable_ambiguities(&executables, &mut diagnostics);
    check_scope(
        &document.declarations,
        "document",
        &index,
        &executables,
        &mut diagnostics,
    );
    diagnostics
}

fn collect_executables<'a>(declarations: &'a [Declaration], index: &mut ExecutableIndex<'a>) {
    for declaration in declarations {
        match declaration {
            Declaration::Action(action) => index
                .actions
                .entry(&action.name.text)
                .or_default()
                .push(action),
            Declaration::Scenario(scenario) => index
                .scenarios
                .entry(&scenario.name.text)
                .or_default()
                .push(scenario),
            _ => {}
        }
        collect_executables(declaration.declarations(), index);
    }
}

fn check_global_executable_ambiguities(
    index: &ExecutableIndex<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut duplicates = Vec::new();
    for (name, actions) in &index.actions {
        if actions.len() > 1 {
            duplicates.push((
                actions[1].name.span.start,
                Diagnostic::new(
                    "MORVA3001",
                    format!("action name '{name}' is globally ambiguous"),
                    actions[1].name.span,
                ),
            ));
        }
    }
    for (name, scenarios) in &index.scenarios {
        if scenarios.len() > 1 {
            duplicates.push((
                scenarios[1].name.span.start,
                Diagnostic::new(
                    "MORVA3002",
                    format!("scenario name '{name}' is globally ambiguous"),
                    scenarios[1].name.span,
                ),
            ));
        }
    }
    duplicates.sort_by_key(|(offset, _)| *offset);
    diagnostics.extend(duplicates.into_iter().map(|(_, diagnostic)| diagnostic));
}

fn check_global_type_ambiguities(index: &TypeIndex<'_>, diagnostics: &mut Vec<Diagnostic>) {
    let mut ambiguous = index
        .declarations
        .iter()
        .filter(|(name, candidates)| candidates.len() > 1 || resolve_builtin(name).is_some())
        .collect::<Vec<_>>();
    ambiguous.sort_by_key(|(_, candidates)| candidates[0].name().span.start);
    for (name, candidates) in ambiguous {
        let message = if resolve_builtin(name).is_some() {
            format!("type name '{name}' conflicts with a built-in type")
        } else {
            format!(
                "type name '{name}' is ambiguous across {} declarations",
                candidates.len()
            )
        };
        diagnostics.push(Diagnostic::new(
            "MORVA2008",
            message,
            candidates[candidates.len().min(2) - 1].name().span,
        ));
    }
}

fn find_nested_systems(
    declarations: &[Declaration],
    at_document: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for declaration in declarations {
        if !at_document && let Declaration::System(system) = declaration {
            diagnostics.push(Diagnostic::new(
                "MORVA2002",
                "system declarations are only allowed at the top level",
                system.name.span,
            ));
        }
        find_nested_systems(declaration.declarations(), false, diagnostics);
    }
}

fn collect_types<'a>(declarations: &'a [Declaration], index: &mut TypeIndex<'a>) {
    for declaration in declarations {
        match declaration {
            Declaration::Entity(entity) => index
                .declarations
                .entry(&entity.name.text)
                .or_default()
                .push(TypeDeclaration::Entity(entity)),
            Declaration::Enum(enumeration) => index
                .declarations
                .entry(&enumeration.name.text)
                .or_default()
                .push(TypeDeclaration::Enum(enumeration)),
            _ => {}
        }
        collect_types(declaration.declarations(), index);
    }
}

fn check_scope(
    declarations: &[Declaration],
    scope: &str,
    index: &TypeIndex<'_>,
    executables: &ExecutableIndex<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut names = HashMap::new();
    for declaration in declarations {
        let name = declaration.name();
        if names.insert(name.text.as_str(), name.span).is_some() {
            diagnostics.push(Diagnostic::new(
                "MORVA2003",
                format!("duplicate declaration '{}' in {scope}", name.text),
                name.span,
            ));
        }
        match declaration {
            Declaration::Entity(entity) => check_entity(entity, index, diagnostics),
            Declaration::Enum(enumeration) => check_enum(enumeration, diagnostics),
            Declaration::Action(action) => check_action(action, index, diagnostics),
            Declaration::Scenario(scenario) => {
                check_scenario(scenario, index, executables, diagnostics)
            }
            Declaration::System(_) | Declaration::Container(_) => {}
        }
        check_scope(
            declaration.declarations(),
            &format!("{} '{}'", declaration.kind(), name.text),
            index,
            executables,
            diagnostics,
        );
    }
}

fn check_scenario(
    scenario: &Scenario,
    types: &TypeIndex<'_>,
    executables: &ExecutableIndex<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let first_scenario_diagnostic = diagnostics.len();
    check_scenario_items(scenario, types, executables, diagnostics);
    diagnostics[first_scenario_diagnostic..].sort_by_key(|diagnostic| diagnostic.span.start);
}

fn check_scenario_items(
    scenario: &Scenario,
    types: &TypeIndex<'_>,
    executables: &ExecutableIndex<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut run_count = 0;
    let mut expect_count = 0;
    let mut seen_run = false;
    let mut seen_expect = false;
    for item in &scenario.items {
        match item {
            ScenarioItem::Given(assignment) => {
                if seen_run || seen_expect {
                    diagnostics.push(Diagnostic::new(
                        "MORVA3003",
                        "given items must appear before run",
                        assignment.span,
                    ));
                }
            }
            ScenarioItem::Run(run) => {
                run_count += 1;
                if seen_run || seen_expect {
                    diagnostics.push(Diagnostic::new(
                        "MORVA3004",
                        "scenario must contain exactly one run before expects",
                        run.span,
                    ));
                }
                seen_run = true;
            }
            ScenarioItem::Expect(expect) => {
                expect_count += 1;
                if !seen_run {
                    diagnostics.push(Diagnostic::new(
                        "MORVA3005",
                        "expect items must appear after run",
                        expect.span,
                    ));
                }
                seen_expect = true;
            }
        }
    }
    if run_count != 1 {
        diagnostics.push(Diagnostic::new(
            "MORVA3004",
            format!("scenario must contain exactly one run, found {run_count}"),
            scenario.name.span,
        ));
    }
    if expect_count == 0 {
        diagnostics.push(Diagnostic::new(
            "MORVA3005",
            "scenario must contain at least one expect",
            scenario.name.span,
        ));
    }
    if run_count != 1 {
        return;
    }
    let run = scenario
        .items
        .iter()
        .find_map(|item| match item {
            ScenarioItem::Run(run) => Some(run),
            _ => None,
        })
        .expect("one run");
    let Some(actions) = executables.actions.get(run.action.text.as_str()) else {
        diagnostics.push(Diagnostic::new(
            "MORVA3006",
            format!("unknown action '{}'", run.action.text),
            run.action.span,
        ));
        return;
    };
    if actions.len() != 1 {
        diagnostics.push(Diagnostic::new(
            "MORVA3006",
            format!("action '{}' is ambiguous", run.action.text),
            run.action.span,
        ));
        return;
    }
    let action = actions[0];
    if run.arguments.len() != action.parameters.len() {
        diagnostics.push(Diagnostic::new(
            "MORVA3007",
            format!(
                "action '{}' expects {} argument(s), found {}",
                action.name.text,
                action.parameters.len(),
                run.arguments.len()
            ),
            run.span,
        ));
        return;
    }
    let mut arguments = HashMap::new();
    let mut runtime_parameters = HashMap::new();
    for (argument, parameter) in run.arguments.iter().zip(&action.parameters) {
        if arguments.contains_key(argument.text.as_str()) {
            diagnostics.push(Diagnostic::new(
                "MORVA3008",
                format!("run argument '{}' must be unique", argument.text),
                argument.span,
            ));
            continue;
        }
        match resolve_type(&parameter.type_name, types) {
            Some(ResolvedType::Entity(entity)) => {
                arguments.insert(argument.text.as_str(), entity);
                runtime_parameters.insert(argument.text.as_str(), parameter);
            }
            _ => diagnostics.push(Diagnostic::new(
                "MORVA3009",
                format!(
                    "action parameter '{}' must have an entity type for simulation",
                    parameter.name.text
                ),
                run.span,
            )),
        }
    }
    let empty_fields = HashMap::new();
    for item in &scenario.items {
        match item {
            ScenarioItem::Given(assignment) => {
                check_given(assignment, &arguments, types, diagnostics)
            }
            ScenarioItem::Expect(expect) => check_predicate(
                expect,
                &empty_fields,
                &runtime_parameters,
                types,
                diagnostics,
            ),
            ScenarioItem::Run(_) => {}
        }
    }
}

fn check_given(
    assignment: &Assignment,
    arguments: &HashMap<&str, &Entity>,
    types: &TypeIndex<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if assignment.operator != AssignmentOperator::Set {
        diagnostics.push(Diagnostic::new(
            "MORVA3010",
            "given only supports '=' initialization",
            assignment.span,
        ));
    }
    if assignment.target.segments.len() != 2 {
        diagnostics.push(Diagnostic::new(
            "MORVA3011",
            "given target must be a run argument field",
            assignment.target.span,
        ));
        return;
    }
    let root = &assignment.target.segments[0];
    let Some(entity) = arguments.get(root.text.as_str()) else {
        diagnostics.push(Diagnostic::new(
            "MORVA3011",
            format!("unknown run argument '{}'", root.text),
            root.span,
        ));
        return;
    };
    let field_name = &assignment.target.segments[1];
    let Some(field) = entity
        .fields
        .iter()
        .find(|field| field.name.text == field_name.text)
    else {
        diagnostics.push(Diagnostic::new(
            "MORVA3011",
            format!(
                "entity '{}' has no field named '{}'",
                entity.name.text, field_name.text
            ),
            field_name.span,
        ));
        return;
    };
    check_scenario_value(&assignment.value, &field.type_name, types, diagnostics);
}

fn check_scenario_value(
    value: &Expr,
    type_name: &Name,
    types: &TypeIndex<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match resolve_type(type_name, types) {
        Some(ResolvedType::Enum(enumeration)) => {
            if let ExprKind::Path(path) = &value.kind
                && path.segments.len() == 1
            {
                check_enum_member(&path.segments[0], enumeration, diagnostics);
            } else {
                unsupported_scenario_value(value, diagnostics);
            }
        }
        Some(ResolvedType::Builtin(BuiltinType::Boolean)) => {
            if !matches!(value.kind, ExprKind::Boolean(_)) {
                unsupported_scenario_value(value, diagnostics);
            }
        }
        Some(ResolvedType::Builtin(BuiltinType::Integer)) => {
            if !matches!(value.kind, ExprKind::Integer(_)) {
                unsupported_scenario_value(value, diagnostics);
            }
        }
        _ => unsupported_scenario_value(value, diagnostics),
    }
}

fn unsupported_scenario_value(value: &Expr, diagnostics: &mut Vec<Diagnostic>) {
    diagnostics.push(Diagnostic::new(
        "MORVA3012",
        "scenario values are limited to enum members, Boolean, and Integer",
        value.span,
    ));
}

fn check_enum(enumeration: &Enum, diagnostics: &mut Vec<Diagnostic>) {
    let mut members = HashMap::new();
    for member in &enumeration.members {
        if members.insert(member.text.as_str(), member.span).is_some() {
            diagnostics.push(Diagnostic::new(
                "MORVA2004",
                format!(
                    "duplicate member '{}' in enum '{}'",
                    member.text, enumeration.name.text
                ),
                member.span,
            ));
        }
    }
}

fn check_entity(entity: &Entity, index: &TypeIndex<'_>, diagnostics: &mut Vec<Diagnostic>) {
    let mut fields = HashMap::new();
    for field in &entity.fields {
        if fields.insert(field.name.text.as_str(), field).is_some() {
            diagnostics.push(Diagnostic::new(
                "MORVA2005",
                format!(
                    "duplicate field '{}' in entity '{}'",
                    field.name.text, entity.name.text
                ),
                field.name.span,
            ));
        }
        check_type_name(&field.type_name, index, diagnostics);
    }
    let parameters = HashMap::new();
    for invariant in &entity.invariants {
        check_predicate(invariant, &fields, &parameters, index, diagnostics);
    }
}

fn check_action(action: &Action, index: &TypeIndex<'_>, diagnostics: &mut Vec<Diagnostic>) {
    let first_action_diagnostic = diagnostics.len();
    let mut parameters = HashMap::new();
    for parameter in &action.parameters {
        if parameters
            .insert(parameter.name.text.as_str(), parameter)
            .is_some()
        {
            diagnostics.push(Diagnostic::new(
                "MORVA2006",
                format!(
                    "duplicate parameter '{}' in action '{}'",
                    parameter.name.text, action.name.text
                ),
                parameter.name.span,
            ));
        }
        check_type_name(&parameter.type_name, index, diagnostics);
    }
    let fields = HashMap::new();
    for clause in &action.clauses {
        for expression in &clause.expressions {
            match expression {
                ClauseExpression::Predicate(expression) => {
                    check_predicate(expression, &fields, &parameters, index, diagnostics)
                }
                ClauseExpression::Assignment(assignment) => {
                    let expected =
                        check_effect_target(&assignment.target, &parameters, index, diagnostics);
                    check_assignment_value(assignment, expected, &parameters, index, diagnostics);
                }
            }
        }
    }
    if diagnostics.len() == first_action_diagnostic {
        check_obvious_action_contradictions(action, &parameters, index, diagnostics);
        diagnostics[first_action_diagnostic..].sort_by_key(|diagnostic| diagnostic.span.start);
    }
}

#[derive(Clone, PartialEq, Eq)]
enum LiteralFactValue {
    Boolean(bool),
    Integer(i64),
    Enum { declaration: usize, member: String },
}

enum LiteralFact {
    Equal(String, LiteralFactValue),
    NotEqual(String, LiteralFactValue),
}

#[derive(Default)]
struct PathConstraints {
    equal: Option<LiteralFactValue>,
    not_equal: Vec<LiteralFactValue>,
}

fn check_obvious_action_contradictions(
    action: &Action,
    parameters: &HashMap<&str, &Parameter>,
    index: &TypeIndex<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut emitted = HashSet::new();
    check_predicate_group(
        action,
        parameters,
        index,
        |kind| matches!(kind, ClauseKind::Requires | ClauseKind::Invariant),
        &mut emitted,
        diagnostics,
    );
    check_predicate_group(
        action,
        parameters,
        index,
        |kind| matches!(kind, ClauseKind::Invariant | ClauseKind::Ensures),
        &mut emitted,
        diagnostics,
    );
    check_final_effect_contradictions(action, parameters, index, diagnostics);
}

fn check_final_effect_contradictions(
    action: &Action,
    parameters: &HashMap<&str, &Parameter>,
    index: &TypeIndex<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut final_facts: HashMap<String, PathConstraints> = HashMap::new();
    for clause in &action.clauses {
        if clause.kind != ClauseKind::Effects {
            continue;
        }
        for expression in &clause.expressions {
            let ClauseExpression::Assignment(assignment) = expression else {
                continue;
            };
            let path = assignment.target.display();
            let value = if assignment.operator == AssignmentOperator::Set {
                resolve_path_without_diagnostics(&assignment.target, parameters, index).and_then(
                    |expected| contextual_literal(&assignment.value, expected, parameters),
                )
            } else {
                None
            };
            match value {
                Some(value) => {
                    final_facts.insert(
                        path,
                        PathConstraints {
                            equal: Some(value),
                            not_equal: Vec::new(),
                        },
                    );
                }
                None => {
                    final_facts.remove(&path);
                }
            }
        }
    }

    for clause in &action.clauses {
        if !matches!(clause.kind, ClauseKind::Invariant | ClauseKind::Ensures) {
            continue;
        }
        for expression in &clause.expressions {
            let ClauseExpression::Predicate(expression) = expression else {
                continue;
            };
            let mut used_paths = Vec::new();
            let value =
                evaluate_formula(expression, &final_facts, parameters, index, &mut used_paths);
            let Some(path) = used_paths.first() else {
                continue;
            };
            let already_reported = diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "MORVA2018" && diagnostic.span == expression.span
            });
            if value == Tri::False && !already_reported {
                diagnostics.push(Diagnostic::new(
                    "MORVA2019",
                    format!("postcondition conflicts with final literal effect for '{path}'"),
                    expression.span,
                ));
            }
        }
    }
}

fn check_predicate_group(
    action: &Action,
    parameters: &HashMap<&str, &Parameter>,
    index: &TypeIndex<'_>,
    include: impl Fn(ClauseKind) -> bool,
    emitted: &mut HashSet<(usize, usize)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut constraints: HashMap<String, PathConstraints> = HashMap::new();
    for clause in &action.clauses {
        if !include(clause.kind) {
            continue;
        }
        for expression in &clause.expressions {
            let ClauseExpression::Predicate(expression) = expression else {
                continue;
            };
            let mut used_paths = Vec::new();
            let value =
                evaluate_formula(expression, &constraints, parameters, index, &mut used_paths);
            if value == Tri::False && emitted.insert((expression.span.start, expression.span.end)) {
                let message = match used_paths.first() {
                    Some(path) => {
                        format!(
                            "predicate conflicts with an earlier literal constraint on '{path}'"
                        )
                    }
                    None => "predicate is always false".to_owned(),
                };
                diagnostics.push(Diagnostic::new("MORVA2018", message, expression.span));
            }
            // Only unconditional top-level exact facts join the group's fact
            // set; facts inside '!' or a '||' branch never leak out.
            match literal_fact(expression, parameters, index) {
                Some(LiteralFact::Equal(path, value)) => {
                    let entry = constraints.entry(path).or_default();
                    if entry.equal.is_none() {
                        entry.equal = Some(value);
                    }
                }
                Some(LiteralFact::NotEqual(path, value)) => {
                    let entry = constraints.entry(path).or_default();
                    if !entry.not_equal.contains(&value) {
                        entry.not_equal.push(value);
                    }
                }
                None => {}
            }
        }
    }
}

/// Conservative three-valued evaluation of a predicate formula against known
/// exact per-path facts. `used_paths` records, in evaluation order, every path
/// whose fact determined a sub-result; an empty list on a False result means
/// the formula is constant-false on its own.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tri {
    True,
    False,
    Unknown,
}

fn evaluate_formula(
    expression: &Expr,
    facts: &HashMap<String, PathConstraints>,
    parameters: &HashMap<&str, &Parameter>,
    index: &TypeIndex<'_>,
    used_paths: &mut Vec<String>,
) -> Tri {
    match &expression.kind {
        ExprKind::Boolean(true) => Tri::True,
        ExprKind::Boolean(false) => Tri::False,
        ExprKind::Integer(_) | ExprKind::Path(_) => Tri::Unknown,
        ExprKind::Not(operand) => {
            match evaluate_formula(operand, facts, parameters, index, used_paths) {
                Tri::True => Tri::False,
                Tri::False => Tri::True,
                Tri::Unknown => Tri::Unknown,
            }
        }
        ExprKind::Or { left, right } => {
            let left = evaluate_formula(left, facts, parameters, index, used_paths);
            if left == Tri::True {
                return Tri::True;
            }
            let right = evaluate_formula(right, facts, parameters, index, used_paths);
            match (left, right) {
                (_, Tri::True) => Tri::True,
                (Tri::False, Tri::False) => Tri::False,
                _ => Tri::Unknown,
            }
        }
        ExprKind::Binary {
            left,
            operator,
            right,
        } => {
            if let (Some(left), Some(right)) = (plain_literal(left), plain_literal(right)) {
                return match evaluate_literal_comparison(left, *operator, right) {
                    Some(true) => Tri::True,
                    Some(false) => Tri::False,
                    None => Tri::Unknown,
                };
            }
            let fact = path_literal_fact(left, *operator, right, parameters, index)
                .or_else(|| path_literal_fact(right, *operator, left, parameters, index));
            let (path, value, expects_equal) = match fact {
                Some(LiteralFact::Equal(path, value)) => (path, value, true),
                Some(LiteralFact::NotEqual(path, value)) => (path, value, false),
                None => return Tri::Unknown,
            };
            let Some(entry) = facts.get(&path) else {
                return Tri::Unknown;
            };
            if let Some(known) = &entry.equal {
                used_paths.push(path.clone());
                let equal = known == &value;
                return if equal == expects_equal {
                    Tri::True
                } else {
                    Tri::False
                };
            }
            if entry.not_equal.contains(&value) {
                used_paths.push(path.clone());
                return if expects_equal { Tri::False } else { Tri::True };
            }
            Tri::Unknown
        }
    }
}

fn literal_fact(
    expression: &Expr,
    parameters: &HashMap<&str, &Parameter>,
    index: &TypeIndex<'_>,
) -> Option<LiteralFact> {
    match &expression.kind {
        ExprKind::Boolean(_) => None,
        ExprKind::Binary {
            left,
            operator,
            right,
        } => {
            if plain_literal(left).is_some() && plain_literal(right).is_some() {
                return None;
            }
            if !matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual) {
                return None;
            }
            path_literal_fact(left, *operator, right, parameters, index)
                .or_else(|| path_literal_fact(right, *operator, left, parameters, index))
        }
        ExprKind::Integer(_) | ExprKind::Path(_) | ExprKind::Not(_) | ExprKind::Or { .. } => None,
    }
}

fn path_literal_fact(
    path_expression: &Expr,
    operator: BinaryOperator,
    literal_expression: &Expr,
    parameters: &HashMap<&str, &Parameter>,
    index: &TypeIndex<'_>,
) -> Option<LiteralFact> {
    let ExprKind::Path(path) = &path_expression.kind else {
        return None;
    };
    let resolved = resolve_path_without_diagnostics(path, parameters, index)?;
    let value = contextual_literal(literal_expression, resolved, parameters)?;
    let path = path.display();
    Some(match operator {
        BinaryOperator::Equal => LiteralFact::Equal(path, value),
        BinaryOperator::NotEqual => LiteralFact::NotEqual(path, value),
        _ => return None,
    })
}

fn resolve_path_without_diagnostics<'a>(
    path: &Path,
    parameters: &HashMap<&str, &Parameter>,
    index: &'a TypeIndex<'a>,
) -> Option<ResolvedType<'a>> {
    let fields = HashMap::new();
    let mut ignored = Vec::new();
    match resolve_path(path, &fields, parameters, index, &mut ignored) {
        Operand::Typed(resolved) if ignored.is_empty() => resolved,
        Operand::Typed(_) | Operand::Unbound(_) => None,
    }
}

fn contextual_literal(
    expression: &Expr,
    expected: ResolvedType<'_>,
    parameters: &HashMap<&str, &Parameter>,
) -> Option<LiteralFactValue> {
    match (&expression.kind, expected) {
        (ExprKind::Boolean(value), ResolvedType::Builtin(BuiltinType::Boolean)) => {
            Some(LiteralFactValue::Boolean(*value))
        }
        (
            ExprKind::Integer(value),
            ResolvedType::Builtin(BuiltinType::Integer | BuiltinType::Decimal),
        ) => Some(LiteralFactValue::Integer(*value)),
        (ExprKind::Path(path), ResolvedType::Enum(enumeration))
            if path.segments.len() == 1
                && !parameters.contains_key(path.segments[0].text.as_str())
                && enumeration
                    .members
                    .iter()
                    .any(|member| member.text == path.segments[0].text) =>
        {
            Some(LiteralFactValue::Enum {
                declaration: enumeration.name.span.start,
                member: path.segments[0].text.clone(),
            })
        }
        _ => None,
    }
}

fn plain_literal(expression: &Expr) -> Option<LiteralFactValue> {
    match expression.kind {
        ExprKind::Boolean(value) => Some(LiteralFactValue::Boolean(value)),
        ExprKind::Integer(value) => Some(LiteralFactValue::Integer(value)),
        ExprKind::Path(_) | ExprKind::Binary { .. } | ExprKind::Not(_) | ExprKind::Or { .. } => {
            None
        }
    }
}

fn evaluate_literal_comparison(
    left: LiteralFactValue,
    operator: BinaryOperator,
    right: LiteralFactValue,
) -> Option<bool> {
    match (left, right) {
        (LiteralFactValue::Boolean(left), LiteralFactValue::Boolean(right)) => match operator {
            BinaryOperator::Equal => Some(left == right),
            BinaryOperator::NotEqual => Some(left != right),
            _ => None,
        },
        (LiteralFactValue::Integer(left), LiteralFactValue::Integer(right)) => {
            Some(match operator {
                BinaryOperator::Equal => left == right,
                BinaryOperator::NotEqual => left != right,
                BinaryOperator::Greater => left > right,
                BinaryOperator::GreaterEqual => left >= right,
                BinaryOperator::Less => left < right,
                BinaryOperator::LessEqual => left <= right,
            })
        }
        _ => None,
    }
}

fn check_type_name(name: &Name, index: &TypeIndex<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if resolve_builtin(&name.text).is_some() {
        return;
    }
    match index.declarations.get(name.text.as_str()) {
        None => diagnostics.push(Diagnostic::new(
            "MORVA2007",
            format!("unknown type '{}'", name.text),
            name.span,
        )),
        Some(candidates) if candidates.len() > 1 => diagnostics.push(Diagnostic::new(
            "MORVA2008",
            format!(
                "type name '{}' is ambiguous across {} declarations",
                name.text,
                candidates.len()
            ),
            name.span,
        )),
        Some(_) => {}
    }
}

fn resolve_type<'a>(name: &Name, index: &'a TypeIndex<'a>) -> Option<ResolvedType<'a>> {
    if let Some(builtin) = resolve_builtin(&name.text) {
        return Some(ResolvedType::Builtin(builtin));
    }
    let candidates = index.declarations.get(name.text.as_str())?;
    if candidates.len() != 1 {
        return None;
    }
    Some(match candidates[0] {
        TypeDeclaration::Entity(entity) => ResolvedType::Entity(entity),
        TypeDeclaration::Enum(enumeration) => ResolvedType::Enum(enumeration),
    })
}

pub(crate) const BUILTIN_TYPE_NAMES: &[&str] = &["Boolean", "Decimal", "ID", "Integer", "String"];

pub(crate) const BUILTIN_TYPE_ALIASES: &[(&str, &str)] =
    &[("Bool", "Boolean"), ("Id", "ID"), ("Int", "Integer")];

fn resolve_builtin(name: &str) -> Option<BuiltinType> {
    match name {
        "Bool" | "Boolean" => Some(BuiltinType::Boolean),
        "Decimal" => Some(BuiltinType::Decimal),
        "ID" | "Id" => Some(BuiltinType::Id),
        "Int" | "Integer" => Some(BuiltinType::Integer),
        "String" => Some(BuiltinType::String),
        _ => None,
    }
}

fn check_predicate(
    expression: &Expr,
    fields: &HashMap<&str, &Field>,
    parameters: &HashMap<&str, &Parameter>,
    index: &TypeIndex<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let ExprKind::Not(operand) = &expression.kind {
        check_predicate(operand, fields, parameters, index, diagnostics);
    } else if let ExprKind::Or { left, right } = &expression.kind {
        check_predicate(left, fields, parameters, index, diagnostics);
        check_predicate(right, fields, parameters, index, diagnostics);
    } else if let ExprKind::Binary {
        left,
        operator,
        right,
    } = &expression.kind
    {
        let left = resolve_operand(left, fields, parameters, index, diagnostics);
        let right = resolve_operand(right, fields, parameters, index, diagnostics);
        match (left, right) {
            (Operand::Unbound(name), Operand::Typed(Some(ResolvedType::Enum(enumeration))))
            | (Operand::Typed(Some(ResolvedType::Enum(enumeration))), Operand::Unbound(name)) => {
                let diagnostic_count = diagnostics.len();
                check_enum_member(name, enumeration, diagnostics);
                if diagnostics.len() == diagnostic_count {
                    check_binary_types(
                        expression,
                        *operator,
                        ResolvedType::Enum(enumeration),
                        ResolvedType::Enum(enumeration),
                        diagnostics,
                    );
                }
            }
            (Operand::Unbound(left), Operand::Unbound(right)) => {
                unknown_reference(left, diagnostics);
                unknown_reference(right, diagnostics);
            }
            (Operand::Unbound(name), _) | (_, Operand::Unbound(name)) => {
                unknown_reference(name, diagnostics)
            }
            (Operand::Typed(Some(left)), Operand::Typed(Some(right))) => {
                check_binary_types(expression, *operator, left, right, diagnostics)
            }
            _ => {}
        }
    } else {
        match resolve_operand(expression, fields, parameters, index, diagnostics) {
            Operand::Unbound(name) => unknown_reference(name, diagnostics),
            Operand::Typed(Some(resolved)) if !is_boolean(resolved) => {
                diagnostics.push(Diagnostic::new(
                    "MORVA2013",
                    format!(
                        "predicate must evaluate to Boolean, found {}",
                        type_display(resolved)
                    ),
                    expression.span,
                ));
            }
            Operand::Typed(_) => {}
        }
    }
}

fn check_binary_types(
    expression: &Expr,
    operator: BinaryOperator,
    left: ResolvedType<'_>,
    right: ResolvedType<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual)
        && (!is_scalar_or_enum(left)
            || (left != right && !has_decimal_integer_constant(expression, left, right)))
    {
        diagnostics.push(Diagnostic::new(
            "MORVA2014",
            format!(
                "operator '{}' requires compatible operand types, found {} and {}",
                binary_operator_display(operator),
                type_display(left),
                type_display(right)
            ),
            expression.span,
        ));
    } else if !matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual)
        && !ordered_types_are_compatible(expression, left, right)
    {
        diagnostics.push(Diagnostic::new(
            "MORVA2015",
            format!(
                "operator '{}' requires Integer or Decimal operands, found {} and {}",
                binary_operator_display(operator),
                type_display(left),
                type_display(right)
            ),
            expression.span,
        ));
    }
}

fn ordered_types_are_compatible(
    expression: &Expr,
    left: ResolvedType<'_>,
    right: ResolvedType<'_>,
) -> bool {
    let same_numeric_type = left == right
        && matches!(
            left,
            ResolvedType::Builtin(BuiltinType::Integer | BuiltinType::Decimal)
        );
    if same_numeric_type {
        return true;
    }
    has_decimal_integer_constant(expression, left, right)
}

fn has_decimal_integer_constant(
    expression: &Expr,
    left: ResolvedType<'_>,
    right: ResolvedType<'_>,
) -> bool {
    let ExprKind::Binary {
        left: left_expression,
        right: right_expression,
        ..
    } = &expression.kind
    else {
        return false;
    };
    matches!(
        (left, right, &left_expression.kind, &right_expression.kind),
        (
            ResolvedType::Builtin(BuiltinType::Decimal),
            ResolvedType::Builtin(BuiltinType::Integer),
            _,
            ExprKind::Integer(_)
        ) | (
            ResolvedType::Builtin(BuiltinType::Integer),
            ResolvedType::Builtin(BuiltinType::Decimal),
            ExprKind::Integer(_),
            _
        )
    )
}

fn is_scalar_or_enum(resolved: ResolvedType<'_>) -> bool {
    matches!(resolved, ResolvedType::Builtin(_) | ResolvedType::Enum(_))
}

fn binary_operator_display(operator: BinaryOperator) -> &'static str {
    match operator {
        BinaryOperator::Equal => "==",
        BinaryOperator::NotEqual => "!=",
        BinaryOperator::Greater => ">",
        BinaryOperator::GreaterEqual => ">=",
        BinaryOperator::Less => "<",
        BinaryOperator::LessEqual => "<=",
    }
}

fn check_assignment_value(
    assignment: &Assignment,
    expected: Option<ResolvedType<'_>>,
    parameters: &HashMap<&str, &Parameter>,
    index: &TypeIndex<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let expression = &assignment.value;
    let fields = HashMap::new();
    if assignment.operator != AssignmentOperator::Set {
        let actual = if matches!(
            expression.kind,
            ExprKind::Binary { .. } | ExprKind::Not(_) | ExprKind::Or { .. }
        ) {
            let diagnostic_count = diagnostics.len();
            check_predicate(expression, &fields, parameters, index, diagnostics);
            if diagnostics.len() != diagnostic_count {
                return;
            }
            ResolvedType::Builtin(BuiltinType::Boolean)
        } else {
            match resolve_operand(expression, &fields, parameters, index, diagnostics) {
                Operand::Unbound(name) => {
                    if let Some(ResolvedType::Enum(enumeration)) = expected {
                        let diagnostic_count = diagnostics.len();
                        check_enum_member(name, enumeration, diagnostics);
                        if diagnostics.len() != diagnostic_count {
                            return;
                        }
                        ResolvedType::Enum(enumeration)
                    } else {
                        unknown_reference(name, diagnostics);
                        return;
                    }
                }
                Operand::Typed(Some(actual)) => actual,
                Operand::Typed(None) => return,
            }
        };
        let Some(expected) = expected else {
            return;
        };
        if !matches!(expected, ResolvedType::Builtin(BuiltinType::Integer))
            || !matches!(actual, ResolvedType::Builtin(BuiltinType::Integer))
        {
            diagnostics.push(Diagnostic::new(
                "MORVA2017",
                format!(
                    "operator '{}' requires Integer target and value, found {} and {}",
                    assignment_operator_display(assignment.operator),
                    type_display(expected),
                    type_display(actual)
                ),
                assignment.span,
            ));
        }
        return;
    }
    if matches!(
        expression.kind,
        ExprKind::Binary { .. } | ExprKind::Not(_) | ExprKind::Or { .. }
    ) {
        let diagnostic_count = diagnostics.len();
        check_predicate(expression, &fields, parameters, index, diagnostics);
        if diagnostics.len() == diagnostic_count
            && let Some(expected) = expected
        {
            check_set_compatibility(
                expression,
                expected,
                ResolvedType::Builtin(BuiltinType::Boolean),
                diagnostics,
            );
        }
        return;
    }
    match resolve_operand(expression, &fields, parameters, index, diagnostics) {
        Operand::Unbound(name) => {
            if let Some(ResolvedType::Enum(enumeration)) = expected {
                check_enum_member(name, enumeration, diagnostics);
            } else {
                unknown_reference(name, diagnostics);
            }
        }
        Operand::Typed(Some(actual)) => {
            if let Some(expected) = expected {
                check_set_compatibility(expression, expected, actual, diagnostics);
            }
        }
        Operand::Typed(None) => {}
    }
}

fn assignment_operator_display(operator: AssignmentOperator) -> &'static str {
    match operator {
        AssignmentOperator::Set => "=",
        AssignmentOperator::Add => "+=",
        AssignmentOperator::Subtract => "-=",
    }
}

fn check_set_compatibility(
    expression: &Expr,
    expected: ResolvedType<'_>,
    actual: ResolvedType<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let decimal_integer_constant = matches!(
        (expected, actual, &expression.kind),
        (
            ResolvedType::Builtin(BuiltinType::Decimal),
            ResolvedType::Builtin(BuiltinType::Integer),
            ExprKind::Integer(_)
        )
    );
    if !decimal_integer_constant && (!is_scalar_or_enum(expected) || expected != actual) {
        diagnostics.push(Diagnostic::new(
            "MORVA2016",
            format!(
                "cannot assign {} to target of type {}",
                type_display(actual),
                type_display(expected)
            ),
            expression.span,
        ));
    }
}

fn resolve_operand<'a, 'b>(
    expression: &'b Expr,
    fields: &HashMap<&str, &Field>,
    parameters: &HashMap<&str, &Parameter>,
    index: &'a TypeIndex<'a>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Operand<'a, 'b> {
    match &expression.kind {
        ExprKind::Integer(_) => Operand::Typed(Some(ResolvedType::Builtin(BuiltinType::Integer))),
        ExprKind::Boolean(_) => Operand::Typed(Some(ResolvedType::Builtin(BuiltinType::Boolean))),
        ExprKind::Path(path) => resolve_path(path, fields, parameters, index, diagnostics),
        ExprKind::Binary { .. } | ExprKind::Not(_) | ExprKind::Or { .. } => Operand::Typed(None),
    }
}

fn is_boolean(resolved: ResolvedType<'_>) -> bool {
    matches!(resolved, ResolvedType::Builtin(BuiltinType::Boolean))
}

fn type_display(resolved: ResolvedType<'_>) -> &str {
    match resolved {
        ResolvedType::Builtin(builtin) => builtin.display(),
        ResolvedType::Entity(entity) => &entity.name.text,
        ResolvedType::Enum(enumeration) => &enumeration.name.text,
    }
}

fn resolve_path<'a, 'b>(
    path: &'b Path,
    fields: &HashMap<&str, &Field>,
    parameters: &HashMap<&str, &Parameter>,
    index: &'a TypeIndex<'a>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Operand<'a, 'b> {
    let root = &path.segments[0];
    let initial = if let Some(parameter) = parameters.get(root.text.as_str()) {
        resolve_type(&parameter.type_name, index)
    } else if let Some(field) = fields.get(root.text.as_str()) {
        resolve_type(&field.type_name, index)
    } else if path.segments.len() == 1 {
        return Operand::Unbound(root);
    } else {
        diagnostics.push(Diagnostic::new(
            "MORVA2009",
            format!("unknown reference '{}'", root.text),
            root.span,
        ));
        return Operand::Typed(None);
    };
    Operand::Typed(resolve_fields(
        initial,
        &path.segments[1..],
        index,
        diagnostics,
    ))
}

fn resolve_fields<'a>(
    mut resolved: Option<ResolvedType<'a>>,
    segments: &[Name],
    index: &'a TypeIndex<'a>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ResolvedType<'a>> {
    for segment in segments {
        let Some(ResolvedType::Entity(entity)) = resolved else {
            diagnostics.push(Diagnostic::new(
                "MORVA2010",
                format!("value has no field named '{}'", segment.text),
                segment.span,
            ));
            return None;
        };
        let Some(field) = entity
            .fields
            .iter()
            .find(|field| field.name.text == segment.text)
        else {
            diagnostics.push(Diagnostic::new(
                "MORVA2010",
                format!(
                    "entity '{}' has no field named '{}'",
                    entity.name.text, segment.text
                ),
                segment.span,
            ));
            return None;
        };
        resolved = resolve_type(&field.type_name, index);
    }
    resolved
}

fn check_effect_target<'a>(
    path: &Path,
    parameters: &HashMap<&str, &Parameter>,
    index: &'a TypeIndex<'a>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ResolvedType<'a>> {
    let root = &path.segments[0];
    let Some(parameter) = parameters.get(root.text.as_str()) else {
        diagnostics.push(Diagnostic::new(
            "MORVA2011",
            format!(
                "effect target '{}' must start with an action parameter",
                path.display()
            ),
            path.span,
        ));
        return None;
    };
    if path.segments.len() < 2 {
        diagnostics.push(Diagnostic::new(
            "MORVA2011",
            format!(
                "effect target '{}' must name a parameter field",
                path.display()
            ),
            path.span,
        ));
        return None;
    }
    resolve_fields(
        resolve_type(&parameter.type_name, index),
        &path.segments[1..],
        index,
        diagnostics,
    )
}

fn check_enum_member(name: &Name, enumeration: &Enum, diagnostics: &mut Vec<Diagnostic>) {
    if !enumeration
        .members
        .iter()
        .any(|member| member.text == name.text)
    {
        diagnostics.push(Diagnostic::new(
            "MORVA2012",
            format!(
                "unknown member '{}' for enum '{}'",
                name.text, enumeration.name.text
            ),
            name.span,
        ));
    }
}

fn unknown_reference(name: &Name, diagnostics: &mut Vec<Diagnostic>) {
    diagnostics.push(Diagnostic::new(
        "MORVA2009",
        format!("unknown reference '{}'", name.text),
        name.span,
    ));
}
