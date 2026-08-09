use std::env;
use std::fs;
use std::process::ExitCode;

use morva_core::{
    Action, AssignmentOperator, BinaryOperator, ClauseExpression, Declaration, Diagnostic,
    Document, Entity, Enum, Expr, ExprKind, check, parse,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [command] if command == "help" || command == "--help" || command == "-h" => {
            help();
            ExitCode::SUCCESS
        }
        [command, path] if command == "check" || command == "parse" || command == "inspect" => {
            run(command, path)
        }
        _ => {
            help();
            ExitCode::from(2)
        }
    }
}

fn run(command: &str, path: &str) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("error: cannot read {path}: {error}");
            return ExitCode::from(2);
        }
    };
    let document = match parse(&source) {
        Ok(document) => document,
        Err(diagnostics) => {
            render_diagnostics(path, &source, &diagnostics);
            return ExitCode::FAILURE;
        }
    };
    let diagnostics = check(&document);
    if !diagnostics.is_empty() {
        render_diagnostics(path, &source, &diagnostics);
        return ExitCode::FAILURE;
    }
    match command {
        "parse" => print_document(&document),
        "inspect" => inspect_document(&document),
        _ => println!("ok: {path}"),
    }
    ExitCode::SUCCESS
}

fn render_diagnostics(path: &str, source: &str, diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        let (line, column, line_start, line_end) = location(source, diagnostic.span.start);
        let source_line = &source.as_bytes()[line_start..line_end];
        let marker_start = diagnostic.span.start.saturating_sub(line_start);
        let marker_end = diagnostic.span.end.saturating_sub(line_start);
        let (rendered, visual_start, visual_len) =
            render_source_line(source_line, marker_start, marker_end);
        eprintln!("{diagnostic}");
        eprintln!("  --> {}:{line}:{column}", safe_path(path));
        eprintln!("   |");
        eprintln!("{line:>3} | {rendered}");
        eprintln!(
            "   | {}{}",
            " ".repeat(visual_start),
            "^".repeat(visual_len)
        );
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

fn location(source: &str, offset: usize) -> (usize, usize, usize, usize) {
    let offset = offset.min(source.len());
    let line_start = source.as_bytes()[..offset]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |index| index + 1);
    let line_end = source.as_bytes()[offset..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(source.len(), |index| offset + index);
    let line = source.as_bytes()[..line_start]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        + 1;
    let column = source[..offset]
        .chars()
        .rev()
        .take_while(|item| *item != '\n')
        .count()
        + 1;
    (line, column, line_start, line_end)
}

fn render_source_line(
    line: &[u8],
    marker_start: usize,
    marker_end: usize,
) -> (String, usize, usize) {
    let marker_start = marker_start.min(line.len());
    let marker_end = marker_end.max(marker_start + 1).min(line.len());
    let mut rendered = String::new();
    let mut visual_start = 0;
    let mut visual_len = 0;
    for (index, byte) in line.iter().copied().enumerate() {
        let fragment = match byte {
            b'\t' => "    ".to_owned(),
            0x20..=0x7e => (byte as char).to_string(),
            _ => format!("\\x{byte:02X}"),
        };
        let width = fragment.len();
        if index < marker_start {
            visual_start += width;
        }
        if index >= marker_start && index < marker_end {
            visual_len += width;
        }
        rendered.push_str(&fragment);
    }
    (rendered, visual_start, visual_len.max(1))
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
        _ => {
            println!("{indent}{} {}", declaration.kind(), declaration.name().text);
            for child in declaration.declarations() {
                print_declaration(child, depth + 1);
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
            let operator = match assignment.operator {
                AssignmentOperator::Set => "=",
                AssignmentOperator::Add => "+=",
                AssignmentOperator::Subtract => "-=",
            };
            format!(
                "{} {operator} {}",
                assignment.target.display(),
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
            let operator = match operator {
                BinaryOperator::Equal => "==",
                BinaryOperator::NotEqual => "!=",
                BinaryOperator::Greater => ">",
                BinaryOperator::GreaterEqual => ">=",
                BinaryOperator::Less => "<",
                BinaryOperator::LessEqual => "<=",
            };
            format!("{} {operator} {}", format_expr(left), format_expr(right))
        }
    }
}

fn inspect_document(document: &Document) {
    let mut enumerations = Vec::new();
    let mut entities = Vec::new();
    let mut actions = Vec::new();
    collect_semantic_items(
        &document.declarations,
        &mut enumerations,
        &mut entities,
        &mut actions,
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
}

fn collect_semantic_items<'a>(
    declarations: &'a [Declaration],
    enumerations: &mut Vec<&'a Enum>,
    entities: &mut Vec<&'a Entity>,
    actions: &mut Vec<&'a Action>,
) {
    for declaration in declarations {
        match declaration {
            Declaration::Enum(enumeration) => enumerations.push(enumeration),
            Declaration::Entity(entity) => entities.push(entity),
            Declaration::Action(action) => actions.push(action),
            _ => {}
        }
        collect_semantic_items(declaration.declarations(), enumerations, entities, actions);
    }
}

fn help() {
    println!(
        "Morva semantic model tools\n\nUsage:\n  morva check <file>\n  morva parse <file>\n  morva inspect <file>\n  morva help"
    );
}
