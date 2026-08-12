use morva_core::{
    Assignment, ClauseExpression, Declaration, Expr, ExprKind, Path, Project, ProjectFinding,
    ScenarioItem, SimulationPhase, SoftBehaviorKind, Span, check, parse, simulate,
};

fn project() -> Project {
    Project::parse([
        (
            "01-types.morva",
            "system Shop {\r\n  enum State {\r\n    Pending\r\n    Confirmed\r\n  }\r\n  entity Order { status: State }\r\n}\r\n",
        ),
        (
            "02-behavior.morva",
            r#"system Shop {
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
        ),
    ])
    .expect("assemble project")
}

#[test]
fn assembles_cross_file_references_and_simulates_seven_phases() {
    let project = project();
    assert!(project.check().is_empty());
    let report = simulate(project.document(), "Happy").expect("select scenario");
    assert!(report.succeeded());
    assert_eq!(report.phases.len(), 7);
    assert_eq!(report.phases[3].phase, SimulationPhase::Effects);
}

#[test]
fn maps_compatibility_notices_to_the_responsible_project_source() {
    let first = "system Shop {\n  module First {}\n}\n";
    let second = "system Shop {\n  module Other {}\n  entity Item { value: Missing }\n}\n";
    let project = Project::parse([
        ("01-compat.morva", first),
        ("02-compat-error.morva", second),
    ])
    .expect("assemble project");
    let report = project.analyze();
    assert_eq!(report.errors, project.check());
    assert_eq!(report.errors[0].diagnostic().code, "MORVA2007");
    assert_eq!(report.notices.len(), 2);
    assert_eq!(report.notices[0].source_id.0, 0);
    assert_eq!(report.notices[1].source_id.0, 1);
    assert_eq!(
        report.notices[0].local_notice.span.start,
        first.find("First").unwrap()
    );
    assert_eq!(
        report.notices[1].local_notice.span.start,
        second.find("Other").unwrap()
    );
    assert_eq!(
        report.notices[0].local_notice.span.start, report.notices[1].local_notice.span.start,
        "equal local offsets must remain associated with distinct sources"
    );
    assert!(matches!(
        report.findings().as_slice(),
        [
            ProjectFinding::Notice(_),
            ProjectFinding::Notice(_),
            ProjectFinding::Error(_)
        ]
    ));
}

#[test]
fn maps_nested_ast_and_runtime_spans_back_to_the_second_source() {
    let project = project();
    let Declaration::System(system) = &project.document().declarations[0] else {
        panic!("merged system")
    };
    let action = system
        .declarations
        .iter()
        .find_map(|item| match item {
            Declaration::Action(action) => Some(action),
            _ => None,
        })
        .expect("action");
    for span in [
        action.span,
        action.name.span,
        action.parameters[0].span,
        action.parameters[0].name.span,
        action.parameters[0].type_name.span,
        action.clauses[0].span,
        action.clauses[0].expressions[0].span(),
    ] {
        let location = project
            .locate_virtual_span(span)
            .expect("mapped nested span");
        assert_eq!(location.source_id.0, 1);
        assert!(location.local_span.end <= project.sources()[1].source.len());
    }

    let runtime_source = "system Shop {\r\n  action Read(order: Order) { requires order.count > 0 }\r\n  scenario Bad {\r\n    given order.count = 0\r\n    run Read(order)\r\n    expect true\r\n  }\r\n}";
    let failing = Project::parse([
        (
            "types.morva",
            "system Shop { entity Order { count: Integer } }",
        ),
        ("behavior.morva", runtime_source),
    ])
    .unwrap();
    let report = simulate(failing.document(), "Bad").unwrap();
    let failure = report.failure.unwrap();
    let location = failing
        .locate_virtual_span(failure.span)
        .expect("mapped runtime span");
    assert_eq!(location.source_id.0, 1);
    let expected_start = runtime_source.find("order.count > 0").unwrap();
    assert_eq!(
        location.local_span,
        morva_core::Span {
            start: expected_start,
            end: expected_start + "order.count > 0".len(),
        }
    );
    assert_eq!(
        &failing.sources()[1].source[location.local_span.start..location.local_span.end],
        "order.count > 0"
    );
}

#[test]
fn preserves_source_order_and_reports_cross_file_duplicates_at_local_spans() {
    let project = Project::parse([
        ("a.morva", "system Shop { enum State { A } }"),
        ("b.morva", "system Shop { enum State { B } }"),
    ])
    .unwrap();
    let diagnostics = project.check();
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics.iter().any(|item| {
        item.source_id() == Some(morva_core::SourceId(1)) && item.diagnostic().span.start == 19
    }));
}

#[test]
fn rejects_mismatched_or_invalid_source_system_shells() {
    let mismatch =
        Project::parse([("a.morva", "system One {}"), ("b.morva", "system Two {}")]).unwrap_err();
    assert_eq!(mismatch[0].source_id().unwrap().0, 1);
    assert_eq!(mismatch[0].diagnostic().code, "MORVA2021");
    assert_eq!(mismatch[0].diagnostic().span.start, 7);

    let extra = Project::parse([("a.morva", "system One {} entity Loose {}")]).unwrap_err();
    assert_eq!(extra[0].diagnostic().code, "MORVA2020");
    assert_eq!(extra[0].diagnostic().span.start, 14);
    assert!(extra[0].diagnostic().message.contains("outside"));

    let missing = Project::parse([("a.morva", "entity Loose {}")]).unwrap_err();
    assert!(missing[0].diagnostic().message.contains("must contain one"));
    assert_eq!(missing[0].diagnostic().span.start, 0);

    let multiple = Project::parse([("a.morva", "system One {}\nsystem One {}")]).unwrap_err();
    assert!(multiple[0].diagnostic().message.contains("multiple"));
    assert_eq!(multiple[0].diagnostic().span.start, 21);
}

#[test]
fn old_single_file_api_remains_exactly_equivalent() {
    let source = include_str!("../../../examples/order.morva");
    let old = parse(source).unwrap();
    let project = Project::parse([("order.morva", source)]).unwrap();
    assert_eq!(old, *project.document());
    assert_eq!(
        check(&old),
        project
            .check()
            .into_iter()
            .map(|item| item.diagnostic().clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        simulate(&old, "NormalConfirmation"),
        simulate(project.document(), "NormalConfirmation")
    );
}

#[test]
fn source_map_rejects_reversed_and_cross_base_spans() {
    let project = project();
    let second_base = project.sources()[0].source.len() + 1;
    assert_eq!(
        project.locate_virtual_span(morva_core::Span { start: 5, end: 4 }),
        None
    );
    assert_eq!(
        project.locate_virtual_span(morva_core::Span {
            start: second_base,
            end: second_base - 1,
        }),
        None
    );
}

#[test]
fn empty_project_has_a_project_level_diagnostic_without_a_source_id() {
    let diagnostics = Project::parse(Vec::<(String, String)>::new()).unwrap_err();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].source_id(), None);
    assert_eq!(diagnostics[0].diagnostic().code, "MORVA2023");
}

#[test]
fn rebases_every_nested_ast_span_variant_and_roundtrips_exactly() {
    let rich_source = r#"system Shop {
  module Domain {
    enum State {
      Ready
      Done
    }
    entity Item {
      count: Integer
      active: Boolean
      state: State
      invariant count >= 0
    }
    action Change(item: Item) {
      atomic
      requires item.active
      invariant item.count >= 0
      effects {
        item.count += 1
        item.state = Done
      }
      ensures item.state == Done
    }
    scenario Happy {
      given item.count = 0
      given item.active = true
      given item.state = Ready
      run Change(item)
      expect item.count == 1
    }
  }
}
"#;
    let local = parse(rich_source).unwrap();
    let project = Project::parse([
        ("00-shell.morva", "system Shop {}\n"),
        ("rich.morva", rich_source),
    ])
    .unwrap();
    let Declaration::System(local_system) = &local.declarations[0] else {
        panic!("local system")
    };
    let Declaration::System(merged_system) = &project.document().declarations[0] else {
        panic!("merged system")
    };
    let mut expected = Vec::new();
    let mut virtual_spans = Vec::new();
    collect_declaration_spans(&local_system.declarations[0], &mut expected);
    collect_declaration_spans(&merged_system.declarations[0], &mut virtual_spans);
    assert_eq!(expected.len(), virtual_spans.len());
    assert!(
        expected.len() > 50,
        "visitor must cover the rich AST deeply"
    );
    for (expected, virtual_span) in expected.into_iter().zip(virtual_spans) {
        let located = project.locate_virtual_span(virtual_span).unwrap();
        assert_eq!(located.source_id.0, 1);
        assert_eq!(located.local_span, expected);
    }
}

fn collect_declaration_spans(declaration: &Declaration, spans: &mut Vec<Span>) {
    spans.push(declaration.span());
    spans.push(declaration.name().span);
    match declaration {
        Declaration::System(item) => {
            for child in &item.declarations {
                collect_declaration_spans(child, spans);
            }
        }
        Declaration::Container(item) => {
            for child in &item.declarations {
                collect_declaration_spans(child, spans);
            }
        }
        Declaration::Entity(item) => {
            for field in &item.fields {
                spans.extend([field.span, field.name.span, field.type_name.span]);
            }
            for invariant in &item.invariants {
                collect_expr_spans(invariant, spans);
            }
        }
        Declaration::Enum(item) => {
            spans.extend(item.members.iter().map(|member| member.span));
        }
        Declaration::Action(item) => {
            for parameter in &item.parameters {
                spans.extend([
                    parameter.span,
                    parameter.name.span,
                    parameter.type_name.span,
                ]);
            }
            for clause in &item.clauses {
                spans.push(clause.span);
                for expression in &clause.expressions {
                    match expression {
                        ClauseExpression::Predicate(expression) => {
                            collect_expr_spans(expression, spans)
                        }
                        ClauseExpression::Assignment(assignment) => {
                            collect_assignment_spans(assignment, spans)
                        }
                    }
                }
            }
            spans.extend(item.soft_behaviors.iter().map(|behavior| behavior.span));
        }
        Declaration::Scenario(item) => {
            for scenario_item in &item.items {
                match scenario_item {
                    ScenarioItem::Given(assignment) => collect_assignment_spans(assignment, spans),
                    ScenarioItem::Run(run) => {
                        spans.extend([run.span, run.action.span]);
                        spans.extend(run.arguments.iter().map(|argument| argument.span));
                    }
                    ScenarioItem::Expect(expression) => collect_expr_spans(expression, spans),
                }
            }
        }
    }
}

fn collect_assignment_spans(assignment: &Assignment, spans: &mut Vec<Span>) {
    spans.push(assignment.span);
    collect_path_spans(&assignment.target, spans);
    collect_expr_spans(&assignment.value, spans);
}

fn collect_expr_spans(expression: &Expr, spans: &mut Vec<Span>) {
    spans.push(expression.span);
    match &expression.kind {
        ExprKind::Path(path) => collect_path_spans(path, spans),
        ExprKind::Binary { left, right, .. } => {
            collect_expr_spans(left, spans);
            collect_expr_spans(right, spans);
        }
        ExprKind::Integer(_) | ExprKind::Boolean(_) => {}
    }
}

fn collect_path_spans(path: &Path, spans: &mut Vec<Span>) {
    spans.push(path.span);
    spans.extend(path.segments.iter().map(|segment| segment.span));
}

#[test]
fn maps_soft_behavior_notices_once_to_distinct_project_sources() {
    let first = "system Shop {\n  action First {\n    atomic\n  }\n}\n";
    let second =
        "system Shop {\n  action Other {\n    retry 2\n  }\n  entity Bad { value: Missing }\n}\n";
    let project = Project::parse([("10-first.morva", first), ("20-second.morva", second)])
        .expect("project parses");
    let report = project.analyze();

    assert_eq!(report.errors, project.check());
    assert_eq!(report.notices.len(), 2);
    assert_eq!(report.notices[0].source_id.0, 0);
    assert_eq!(report.notices[1].source_id.0, 1);
    assert_eq!(
        report.notices[0].local_notice.span.start,
        first.find("atomic").unwrap()
    );
    assert_eq!(
        report.notices[1].local_notice.span.start,
        second.find("retry").unwrap()
    );
    assert_eq!(
        report.notices[0].local_notice.span.start, report.notices[1].local_notice.span.start,
        "equal local offsets must remain bound to their own sources"
    );
    assert_eq!(
        report.notices[0].local_notice.kind,
        morva_core::NoticeKind::ActionSoftBehavior {
            action: "First".to_owned(),
            behavior: SoftBehaviorKind::Atomic,
        }
    );
    assert_eq!(
        report.notices[1].local_notice.kind,
        morva_core::NoticeKind::ActionSoftBehavior {
            action: "Other".to_owned(),
            behavior: SoftBehaviorKind::Retry,
        }
    );
    assert!(matches!(
        report.findings().as_slice(),
        [
            ProjectFinding::Notice(_),
            ProjectFinding::Notice(_),
            ProjectFinding::Error(_)
        ]
    ));
}
