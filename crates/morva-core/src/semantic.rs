use std::collections::HashMap;

use crate::Diagnostic;
use crate::ast::*;

const BUILTIN_TYPES: &[&str] = &[
    "Bool", "Boolean", "Decimal", "ID", "Id", "Int", "Integer", "String",
];

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

#[derive(Clone, Copy)]
enum ResolvedType<'a> {
    Builtin,
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
    check_scope(&document.declarations, "document", &index, &mut diagnostics);
    diagnostics
}

fn check_global_type_ambiguities(index: &TypeIndex<'_>, diagnostics: &mut Vec<Diagnostic>) {
    let mut ambiguous = index
        .declarations
        .iter()
        .filter(|(name, candidates)| candidates.len() > 1 || BUILTIN_TYPES.contains(name))
        .collect::<Vec<_>>();
    ambiguous.sort_by_key(|(_, candidates)| candidates[0].name().span.start);
    for (name, candidates) in ambiguous {
        let message = if BUILTIN_TYPES.contains(name) {
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
            Declaration::System(_) | Declaration::Container(_) => {}
        }
        check_scope(
            declaration.declarations(),
            &format!("{} '{}'", declaration.kind(), name.text),
            index,
            diagnostics,
        );
    }
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
                    check_assignment_value(
                        &assignment.value,
                        expected,
                        &parameters,
                        index,
                        diagnostics,
                    );
                }
            }
        }
    }
}

fn check_type_name(name: &Name, index: &TypeIndex<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if BUILTIN_TYPES.contains(&name.text.as_str()) {
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
    if BUILTIN_TYPES.contains(&name.text.as_str()) {
        return Some(ResolvedType::Builtin);
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

fn check_predicate(
    expression: &Expr,
    fields: &HashMap<&str, &Field>,
    parameters: &HashMap<&str, &Parameter>,
    index: &TypeIndex<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let ExprKind::Binary { left, right, .. } = &expression.kind {
        let left = resolve_operand(left, fields, parameters, index, diagnostics);
        let right = resolve_operand(right, fields, parameters, index, diagnostics);
        match (left, right) {
            (Operand::Unbound(name), Operand::Typed(Some(ResolvedType::Enum(enumeration))))
            | (Operand::Typed(Some(ResolvedType::Enum(enumeration))), Operand::Unbound(name)) => {
                check_enum_member(name, enumeration, diagnostics)
            }
            (Operand::Unbound(left), Operand::Unbound(right)) => {
                unknown_reference(left, diagnostics);
                unknown_reference(right, diagnostics);
            }
            (Operand::Unbound(name), _) | (_, Operand::Unbound(name)) => {
                unknown_reference(name, diagnostics)
            }
            _ => {}
        }
    } else if let Operand::Unbound(name) =
        resolve_operand(expression, fields, parameters, index, diagnostics)
    {
        unknown_reference(name, diagnostics);
    }
}

fn check_assignment_value(
    expression: &Expr,
    expected: Option<ResolvedType<'_>>,
    parameters: &HashMap<&str, &Parameter>,
    index: &TypeIndex<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let fields = HashMap::new();
    if matches!(expression.kind, ExprKind::Binary { .. }) {
        check_predicate(expression, &fields, parameters, index, diagnostics);
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
        Operand::Typed(_) => {}
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
        ExprKind::Integer(_) | ExprKind::Boolean(_) => Operand::Typed(Some(ResolvedType::Builtin)),
        ExprKind::Path(path) => resolve_path(path, fields, parameters, index, diagnostics),
        ExprKind::Binary { .. } => Operand::Typed(None),
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
