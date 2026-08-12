use morva_core::{
    AnalysisFinding, ClauseExpression, ClauseKind, Declaration, NoticeKind, SoftBehaviorKind,
    analyze, check, parse,
};

const COMPLETE_MODEL: &str = r#"system Wallet {
  enum AccountState {
    Open
    Frozen
  }
  entity Account {
    id: ID
    balance: Decimal
    state: AccountState
    invariant balance >= 0
  }
  action Freeze(account: Account) {
    requires {
      account.state == Open
    }
    effects {
      account.state = Frozen
    }
    ensures {
      Frozen == account.state
    }
  }
}
"#;

fn codes(source: &str) -> Vec<&'static str> {
    let document = parse(source).expect("valid syntax");
    check(&document).into_iter().map(|item| item.code).collect()
}

#[test]
fn compatibility_containers_are_non_fatal_structured_notices() {
    let source = r#"system Shop {
  module Modules {
    service Services {
      event Events {
        flow Flows {
          lifecycle Lifecycles {
            policy Policies {
              compatibility_only content
            }
          }
        }
      }
    }
  }
}
"#;
    let document = parse(source).expect("compatibility containers parse");
    let report = analyze(&document);
    assert!(report.errors.is_empty());
    assert_eq!(report.notices.len(), 6);
    assert!(check(&document).is_empty());
    for (notice, (kind, name)) in report.notices.iter().zip([
        ("module", "Modules"),
        ("service", "Services"),
        ("event", "Events"),
        ("flow", "Flows"),
        ("lifecycle", "Lifecycles"),
        ("policy", "Policies"),
    ]) {
        assert_eq!(notice.code, "MORVA5001");
        assert_eq!(
            notice.message,
            format!("compatibility {kind} '{name}' is parsed but not semantically validated")
        );
        assert_eq!(
            notice.span.start,
            source.find(name).expect("container name span")
        );
        assert_eq!(notice.span.end - notice.span.start, name.len());
        assert_eq!(
            notice.kind,
            NoticeKind::CompatibilityContainer {
                kind: kind.to_owned(),
                name: name.to_owned(),
            }
        );
    }
}

#[test]
fn action_soft_behaviors_keep_only_kind_and_keyword_span_in_source_order() {
    let source = r#"system Shop {
  entity Order { id: ID }
  action Save(order: Order) {
    atomic retry 99
    idempotent key order.id
    timeout 30
    retry 2
    implementation_hint {
      adapter { nested value }
    }
  }
}
"#;
    let document = parse(source).expect("soft behaviors parse");
    let Declaration::System(system) = &document.declarations[0] else {
        panic!("system")
    };
    let action = system
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::Action(action) => Some(action),
            _ => None,
        })
        .expect("action");
    assert_eq!(
        action
            .soft_behaviors
            .iter()
            .map(|item| item.kind)
            .collect::<Vec<_>>(),
        [
            SoftBehaviorKind::Atomic,
            SoftBehaviorKind::Idempotent,
            SoftBehaviorKind::Timeout,
            SoftBehaviorKind::Retry,
            SoftBehaviorKind::ImplementationHint,
        ]
    );
    for item in &action.soft_behaviors {
        let keyword = item.kind.as_str();
        assert_eq!(&source[item.span.start..item.span.end], keyword);
    }
    assert_eq!(
        action
            .soft_behaviors
            .iter()
            .filter(|item| item.kind == SoftBehaviorKind::Atomic)
            .count(),
        1,
        "a whitelisted word in opaque payload is not another item"
    );
    let report = analyze(&document);
    assert!(report.errors.is_empty());
    assert_eq!(report.notices.len(), 5);
    for (notice, behavior) in report.notices.iter().zip([
        SoftBehaviorKind::Atomic,
        SoftBehaviorKind::Idempotent,
        SoftBehaviorKind::Timeout,
        SoftBehaviorKind::Retry,
        SoftBehaviorKind::ImplementationHint,
    ]) {
        assert_eq!(notice.code, "MORVA5002");
        assert_eq!(
            notice.message,
            format!(
                "action 'Save' soft behavior '{}' is parsed but not semantically validated or executed by simulation",
                behavior.as_str()
            )
        );
        assert_eq!(
            notice.kind,
            NoticeKind::ActionSoftBehavior {
                action: "Save".to_owned(),
                behavior,
            }
        );
    }
}

#[test]
fn action_soft_behaviors_are_non_fatal_structured_notices() {
    let source = "system Shop {\n  action Save {\n    retry 2\n    atomic\n  }\n}\n";
    let document = parse(source).expect("soft behaviors parse");
    let report = analyze(&document);
    assert!(report.errors.is_empty());
    assert!(check(&document).is_empty());
    assert_eq!(report.notices.len(), 2);

    for (notice, behavior) in report
        .notices
        .iter()
        .zip([SoftBehaviorKind::Retry, SoftBehaviorKind::Atomic])
    {
        let keyword = behavior.as_str();
        assert_eq!(notice.code, "MORVA5002");
        assert_eq!(
            notice.message,
            format!(
                "action 'Save' soft behavior '{keyword}' is parsed but not semantically validated or executed by simulation"
            )
        );
        assert_eq!(&source[notice.span.start..notice.span.end], keyword);
        assert_eq!(
            notice.kind,
            NoticeKind::ActionSoftBehavior {
                action: "Save".to_owned(),
                behavior,
            }
        );
    }
}

#[test]
fn analysis_keeps_compatibility_notices_separate_from_semantic_errors() {
    let document =
        parse("system Shop {\n  module Orders {}\n  entity Item { value: Missing }\n}\n")
            .expect("syntax remains valid");
    let report = analyze(&document);
    assert_eq!(report.notices.len(), 1);
    assert_eq!(report.errors, check(&document));
    assert_eq!(report.errors[0].code, "MORVA2007");

    let duplicate = parse("system Shop {\n  module Orders {}\n  module Orders {}\n}\n")
        .expect("duplicate container names remain syntactically valid");
    let duplicate_report = analyze(&duplicate);
    assert!(matches!(
        duplicate_report.findings().as_slice(),
        [
            AnalysisFinding::Notice(_),
            AnalysisFinding::Error(_),
            AnalysisFinding::Notice(_)
        ]
    ));
}

#[test]
fn parses_the_complete_strongly_typed_core() {
    let document = parse(COMPLETE_MODEL).expect("valid model");
    assert!(check(&document).is_empty());
    assert!(analyze(&document).notices.is_empty());
    let Declaration::System(system) = &document.declarations[0] else {
        panic!("expected system");
    };
    let Declaration::Enum(enumeration) = &system.declarations[0] else {
        panic!("expected enum");
    };
    assert_eq!(
        enumeration
            .members
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>(),
        ["Open", "Frozen"]
    );
    let Declaration::Entity(entity) = &system.declarations[1] else {
        panic!("expected entity");
    };
    assert_eq!(entity.fields.len(), 3);
    assert_eq!(entity.invariants.len(), 1);
    let Declaration::Action(action) = &system.declarations[2] else {
        panic!("expected action");
    };
    assert_eq!(action.parameters.len(), 1);
    assert_eq!(
        action
            .clauses
            .iter()
            .map(|item| item.kind)
            .collect::<Vec<_>>(),
        [
            ClauseKind::Requires,
            ClauseKind::Effects,
            ClauseKind::Ensures
        ]
    );
    assert!(matches!(
        action.clauses[1].expressions[0],
        ClauseExpression::Assignment(_)
    ));
}

#[test]
fn existing_example_remains_valid() {
    let document = parse(include_str!("../../../examples/order.morva")).expect("example parses");
    assert!(check(&document).is_empty());
}

#[test]
fn cr_only_newlines_separate_language_items_and_preserve_byte_spans() {
    let source = "system Shop {\r  enum State {\r    Open\r    Closed\r  }\r}\r";
    let document = parse(source).expect("CR-only model parses");
    assert!(check(&document).is_empty());
    let Declaration::System(system) = &document.declarations[0] else {
        panic!("expected system");
    };
    let Declaration::Enum(enumeration) = &system.declarations[0] else {
        panic!("expected enum");
    };
    assert_eq!(
        enumeration
            .members
            .iter()
            .map(|member| member.text.as_str())
            .collect::<Vec<_>>(),
        ["Open", "Closed"]
    );
    let closed_start = source.find("Closed").expect("Closed byte offset");
    assert_eq!(enumeration.members[1].span.start, closed_start);
    assert_eq!(
        enumeration.members[1].span.end,
        closed_start + "Closed".len()
    );
}

#[test]
fn line_comments_stop_before_every_supported_newline_sequence() {
    for newline in ["\n", "\r\n", "\r"] {
        let source = [
            "system Shop {",
            newline,
            "  // retain the next declaration",
            newline,
            "  enum State { Open }",
            newline,
            "}",
            newline,
        ]
        .concat();
        let document = parse(&source).expect("comment ends at logical newline");
        assert!(check(&document).is_empty());
        let Declaration::System(system) = &document.declarations[0] else {
            panic!("expected system");
        };
        assert!(matches!(system.declarations[0], Declaration::Enum(_)));
    }
}

#[test]
fn block_comments_between_tokens_do_not_change_the_ast() {
    let commented = "system /* system name follows */ Shop { entity Order { status: /* field type */ Boolean } }";
    let document = parse(commented).unwrap();
    assert!(check(&document).is_empty());
    let Declaration::System(system) = &document.declarations[0] else {
        panic!("system")
    };
    let Declaration::Entity(entity) = &system.declarations[0] else {
        panic!("entity")
    };
    assert_eq!(system.name.text, "Shop");
    assert_eq!(entity.name.text, "Order");
    assert_eq!(entity.fields[0].name.text, "status");
    assert_eq!(entity.fields[0].type_name.text, "Boolean");
}

#[test]
fn block_comments_nest_and_keep_comment_modes_isolated() {
    let source = r#"/* outer // stays block
   /* inner */ outer */
system Shop { // line mode treats /* as text
  entity Order { active: Boolean }
}
"#;
    let document = parse(source).expect("nested and mode-isolated comments parse");
    assert!(check(&document).is_empty());
}

#[test]
fn block_comment_newlines_separate_syntax_and_preserve_original_byte_spans() {
    for newline in ["\n", "\r\n", "\r"] {
        let source = format!(
            "system Shop {{{newline}  enum State {{{newline}    Ready/* note{newline}still note */Done{newline}  }}{newline}}}{newline}"
        );
        let document = parse(&source).expect("comment newline separates enum members");
        let Declaration::System(system) = &document.declarations[0] else {
            panic!("system")
        };
        let Declaration::Enum(enumeration) = &system.declarations[0] else {
            panic!("enum")
        };
        assert_eq!(
            enumeration
                .members
                .iter()
                .map(|member| member.text.as_str())
                .collect::<Vec<_>>(),
            ["Ready", "Done"]
        );
        let done = source.find("Done").unwrap();
        assert_eq!(enumeration.members[1].span.start, done);
        assert_eq!(enumeration.members[1].span.end, done + 4);
    }
}

#[test]
fn unterminated_nested_block_comment_reports_only_the_outer_opener() {
    let source = "system Shop {\r\n  /* outer\n     /* inner still open";
    let diagnostics = parse(source).unwrap_err();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "MORVA1024");
    assert_eq!(diagnostics[0].message, "unterminated block comment");
    let start = source.find("/* outer").unwrap();
    assert_eq!(diagnostics[0].span.start, start);
    assert_eq!(diagnostics[0].span.end, start + 2);
}

#[test]
fn mixed_newlines_inside_block_comments_keep_following_diagnostic_spans_original() {
    let source =
        "system Shop {/* first\nsecond\r\nthird\rfour */\n  entity Item { active Boolean }\n}";
    let diagnostics = parse(source).unwrap_err();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "MORVA1006");
    let boolean = source.find("Boolean").unwrap();
    assert_eq!(diagnostics[0].span.start, boolean);
    assert_eq!(diagnostics[0].span.end, boolean + "Boolean".len());
}

#[test]
fn comments_cannot_split_an_identifier_token() {
    let source = "system Shop { entity Or/* no */der {} }";
    let diagnostics = parse(source).unwrap_err();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "MORVA1025");
    assert_eq!(diagnostics[0].message, "comment cannot split a token");
    let opener = source.find("/*").unwrap();
    assert_eq!(diagnostics[0].span.start, opener);
    assert_eq!(diagnostics[0].span.end, opener + 2);
}

#[test]
fn token_split_comments_are_rejected_in_skipped_and_consecutive_content() {
    for source in [
        "system Sh/**//**/op {}",
        "system Shop { action Save { implementation_hint { storage: rela/**/tional } } }",
        "system Shop { action Save { timeout 1/**/0 } }",
        "system Shop { module Compat { custom Or/**/der } }",
    ] {
        let diagnostics = parse(source).unwrap_err();
        assert_eq!(diagnostics.len(), 1, "{source}");
        assert_eq!(diagnostics[0].code, "MORVA1025", "{source}");
        assert_eq!(diagnostics[0].message, "comment cannot split a token");
        let opener = source.find("/*").unwrap();
        assert_eq!(
            diagnostics[0].span,
            morva_core::Span {
                start: opener,
                end: opener + 2
            }
        );
    }
}

#[test]
fn comments_cannot_split_existing_compound_operators() {
    for (operator, clause) in [
        ("=/**/=", "requires item.count =/**/= 0"),
        ("!/**/=", "requires item.count !/**/= 0"),
        (">/**/=", "requires item.count >/**/= 0"),
        ("</**/=", "requires item.count </**/= 0"),
        ("+/**/=", "effects item.count +/**/= 1"),
        ("-/**/=", "effects item.count -/**/= 1"),
    ] {
        let source = format!(
            "system Test {{ entity Item {{ count: Integer }} action Check(item: Item) {{ {clause} }} }}"
        );
        let diagnostics = parse(&source).unwrap_err();
        assert_eq!(diagnostics.len(), 1, "{operator}");
        assert_eq!(diagnostics[0].code, "MORVA1025", "{operator}");
        assert_eq!(diagnostics[0].message, "comment cannot split a token");
        let opener = source.find("/**/").unwrap();
        assert_eq!(
            diagnostics[0].span,
            morva_core::Span {
                start: opener,
                end: opener + 2,
            },
            "{operator}"
        );
    }
}

#[test]
fn compound_operator_split_guard_covers_skipped_and_consecutive_comments() {
    for split_operator in [
        "=/**/=",
        "!/**/=",
        ">/**/=",
        "</**/=",
        "+/**/=",
        "-/**/=",
        "=/**//**/=",
    ] {
        let source = format!(
            "system Shop {{ action Save {{ implementation_hint {{ operator: {split_operator} }} }} }}"
        );
        let diagnostics = parse(&source).unwrap_err();
        assert_eq!(diagnostics.len(), 1, "{split_operator}");
        assert_eq!(diagnostics[0].code, "MORVA1025", "{split_operator}");
        assert_eq!(diagnostics[0].message, "comment cannot split a token");
        let opener = source.find("/*").unwrap();
        assert_eq!(
            diagnostics[0].span,
            morva_core::Span {
                start: opener,
                end: opener + 2,
            },
            "{split_operator}"
        );
    }
}

#[test]
fn newline_inside_operator_comment_is_a_separator_not_a_token_split() {
    let skipped = "system Shop { action Save { implementation_hint { operator: =/* note\n*/= } } }";
    assert!(parse(skipped).is_ok());

    let typed = "system Test { entity Item { count: Integer } action Check(item: Item) { requires item.count =/* note\n*/= 0 } }";
    let diagnostics = parse(typed).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "MORVA1025")
    );
}

#[test]
fn newline_comments_and_non_joining_boundaries_do_not_report_token_splits() {
    let source = "system Shop { action Save { implementation_hint { storage: rela/* note\n*/tional count: 1/**/value } } }";
    let document = parse(source).expect("comment boundaries do not form one existing token");
    assert!(check(&document).is_empty());
}

#[test]
fn line_comment_at_eof_remains_valid() {
    let document = parse("system Shop {} // trailing comment at EOF").unwrap();
    assert!(check(&document).is_empty());
}

#[test]
fn equivalent_newline_sequences_keep_model_shape_and_original_byte_spans() {
    for newline in ["\n", "\r\n", "\r"] {
        let source = [
            "system Shop {",
            newline,
            "  enum State {",
            newline,
            "    Open",
            newline,
            "    Closed",
            newline,
            "  }",
            newline,
            "}",
            newline,
        ]
        .concat();
        let document = parse(&source).expect("equivalent model parses");
        assert!(check(&document).is_empty());
        assert_eq!(document.span.end, source.len());
        let Declaration::System(system) = &document.declarations[0] else {
            panic!("expected system");
        };
        let Declaration::Enum(enumeration) = &system.declarations[0] else {
            panic!("expected enum");
        };
        assert_eq!(
            enumeration
                .members
                .iter()
                .map(|member| member.text.as_str())
                .collect::<Vec<_>>(),
            ["Open", "Closed"]
        );
        for member in &enumeration.members {
            let start = source.find(&member.text).expect("member byte offset");
            assert_eq!(member.span.start, start);
            assert_eq!(member.span.end, start + member.text.len());
        }
    }
}

#[test]
fn mixed_newline_sequences_are_single_logical_separators() {
    let source =
        "system Shop {\r  // mixed source\r\n  enum State {\n    Open\r    Closed\r\n  }\r}\n";
    let document = parse(source).expect("mixed-newline model parses");
    assert!(check(&document).is_empty());
    let Declaration::System(system) = &document.declarations[0] else {
        panic!("expected system");
    };
    let Declaration::Enum(enumeration) = &system.declarations[0] else {
        panic!("expected enum");
    };
    assert_eq!(enumeration.members.len(), 2);
    assert_eq!(
        enumeration.members[1].span.start,
        source.find("Closed").expect("Closed byte offset")
    );
}

#[test]
fn enum_members_require_the_expected_enum_context() {
    let source = r#"system Shop {
  enum OrderStatus { Pending }
  enum PaymentStatus { Paid }
  entity Order { status: OrderStatus }
  action Check(order: Order) {
    requires order.status == Paid
    effects order.status = Missing
  }
}
"#;
    let diagnostics = check(&parse(source).expect("valid syntax"));
    assert_eq!(
        diagnostics
            .iter()
            .filter(|item| item.code == "MORVA2012")
            .count(),
        2
    );
    assert!(diagnostics.iter().any(|item| item.message.contains("Paid")));
    assert!(
        diagnostics
            .iter()
            .any(|item| item.message.contains("Missing"))
    );
    assert!(diagnostics.iter().all(|item| item.code == "MORVA2012"));
}

#[test]
fn unknown_bare_and_dotted_references_are_rejected() {
    let source = r#"system Shop {
  action Check {
    requires Missing
    ensures unknown.field == true
  }
}
"#;
    assert_eq!(
        codes(source)
            .into_iter()
            .filter(|code| *code == "MORVA2009")
            .count(),
        2
    );
}

#[test]
fn duplicate_names_and_unknown_types_are_reported() {
    let source = r#"system Shop {
  enum State {
    Open
    Open
  }
  entity Order {
    id: Missing
    id: ID
  }
  entity Order {}
  action Use(x: Order, x: Order) {}
}
"#;
    let diagnostics = check(&parse(source).expect("valid syntax"));
    for code in [
        "MORVA2003",
        "MORVA2004",
        "MORVA2005",
        "MORVA2006",
        "MORVA2007",
    ] {
        assert!(
            diagnostics.iter().any(|item| item.code == code),
            "missing {code}: {diagnostics:?}"
        );
    }
}

#[test]
fn effects_must_write_a_parameter_field() {
    let source = r#"system Shop {
  entity Order { status: String }
  action Change(order: Order) {
    effects order = true
    effects external.status = true
  }
}
"#;
    assert_eq!(
        codes(source)
            .into_iter()
            .filter(|code| *code == "MORVA2011")
            .count(),
        2
    );
}

#[test]
fn predicates_must_be_boolean() {
    let source = "system Test {\n  entity Item {\n    count: Integer\n    invariant count\n  }\n}";
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    let diagnostic = diagnostics
        .iter()
        .find(|item| item.code == "MORVA2013")
        .expect("non-Boolean predicate must be rejected");
    assert_eq!(
        diagnostic.message,
        "predicate must evaluate to Boolean, found Integer"
    );
    let start = source.rfind("invariant count").expect("predicate location") + "invariant ".len();
    assert_eq!(diagnostic.span.start, start);
    assert_eq!(diagnostic.span.end, start + "count".len());
}

#[test]
fn boolean_literals_and_paths_are_valid_predicates() {
    let source = r#"system Test {
  entity Item {
    enabled: Boolean
    invariant enabled
  }
  action Observe(item: Item) {
    requires true
    ensures item.enabled
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    assert!(check(&document).is_empty());
}

#[test]
fn equality_requires_matching_canonical_types() {
    let source = r#"system Test {
  entity Item {
    count: Integer
    enabled: Boolean
    invariant count == enabled
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    let diagnostic = diagnostics
        .iter()
        .find(|item| item.code == "MORVA2014")
        .expect("mixed equality must be rejected");
    assert_eq!(
        diagnostic.message,
        "operator '==' requires compatible operand types, found Integer and Boolean"
    );
    let start = source
        .find("count == enabled")
        .expect("comparison location");
    assert_eq!(diagnostic.span.start, start);
    assert_eq!(diagnostic.span.end, start + "count == enabled".len());
}

#[test]
fn inequality_accepts_matching_canonical_types() {
    let source = r#"system Test {
  entity Item {
    old_count: Int
    new_count: Integer
    invariant old_count != new_count
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    assert!(check(&document).is_empty());
}

#[test]
fn ordered_comparisons_require_integer_or_decimal_operands() {
    let source = r#"system Test {
  entity Item {
    enabled: Boolean
    invariant enabled > false
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    let diagnostic = diagnostics
        .iter()
        .find(|item| item.code == "MORVA2015")
        .expect("Boolean ordering must be rejected");
    assert_eq!(
        diagnostic.message,
        "operator '>' requires Integer or Decimal operands, found Boolean and Boolean"
    );
    let start = source
        .find("enabled > false")
        .expect("ordered comparison location");
    assert_eq!(diagnostic.span.start, start);
    assert_eq!(diagnostic.span.end, start + "enabled > false".len());
}

#[test]
fn set_effects_require_a_compatible_value_type() {
    let source = r#"system Test {
  entity Counter { count: Integer }
  action Corrupt(counter: Counter) {
    effects counter.count = true
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    let diagnostic = diagnostics
        .iter()
        .find(|item| item.code == "MORVA2016")
        .expect("incompatible effect value must be rejected");
    assert_eq!(
        diagnostic.message,
        "cannot assign Boolean to target of type Integer"
    );
    let start = source.find("true").expect("value location");
    assert_eq!(diagnostic.span.start, start);
    assert_eq!(diagnostic.span.end, start + "true".len());
}

#[test]
fn compound_effects_require_integer_target_and_value() {
    let source = r#"system Test {
  entity Switch { enabled: Boolean }
  action Corrupt(switch: Switch) {
    effects switch.enabled += 1
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    let matching = diagnostics
        .iter()
        .filter(|item| item.code == "MORVA2017")
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1);
    assert_eq!(
        matching[0].message,
        "operator '+=' requires Integer target and value, found Boolean and Integer"
    );
    let start = source
        .find("switch.enabled += 1")
        .expect("compound assignment location");
    assert_eq!(matching[0].span.start, start);
    assert_eq!(matching[0].span.end, start + "switch.enabled += 1".len());
}

#[test]
fn integer_add_and_subtract_effects_are_valid() {
    let source = r#"system Test {
  entity Counter { count: Integer }
  action Adjust(counter: Counter) {
    effects counter.count += 1
    effects counter.count -= 1
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    assert!(check(&document).is_empty());
}

#[test]
fn compound_effects_reject_boolean_binary_values_once() {
    let source = r#"system Test {
  entity Counter { count: Integer }
  action Corrupt(counter: Counter) {
    effects counter.count += counter.count == 0
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert_eq!(
        diagnostics
            .iter()
            .filter(|item| item.code == "MORVA2017")
            .count(),
        1
    );
}

#[test]
fn compound_effects_report_enum_values_as_type_errors() {
    let source = r#"system Test {
  enum State { Ready }
  entity Item { state: State }
  action Corrupt(item: Item) {
    effects item.state += Ready
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "MORVA2017");
}

#[test]
fn ordered_enum_comparisons_are_rejected_with_contextual_members() {
    let source = r#"system Test {
  enum State { Ready }
  entity Item {
    state: State
    invariant state > Ready
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "MORVA2015");
}

#[test]
fn builtin_aliases_share_canonical_types() {
    let source = r#"system Test {
  entity Item {
    old_flag: Bool
    new_flag: Boolean
    old_count: Int
    new_count: Integer
    old_id: ID
    new_id: Id
  }
  action Copy(item: Item) {
    requires item.old_flag == item.new_flag
    requires item.old_count == item.new_count
    requires item.old_id == item.new_id
    effects item.new_flag = item.old_flag
    effects item.new_count = item.old_count
    effects item.new_id = item.old_id
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    assert!(check(&document).is_empty());
}

#[test]
fn distinct_builtin_families_remain_incompatible() {
    let source = r#"system Test {
  entity Item {
    id: Id
    name: String
    invariant id == name
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "MORVA2014");
    assert_eq!(
        diagnostics[0].message,
        "operator '==' requires compatible operand types, found ID and String"
    );
}

#[test]
fn decimal_context_accepts_integer_constants_but_not_integer_paths() {
    let source = r#"system Test {
  entity Account {
    balance: Decimal
    count: Integer
    invariant balance >= 0
    invariant 0 <= balance
    invariant balance == 0
    invariant balance > count
  }
  action Reset(account: Account) {
    effects account.balance = 0
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert_eq!(
        diagnostics
            .iter()
            .filter(|item| item.code == "MORVA2015")
            .count(),
        1
    );
    assert!(!diagnostics.iter().any(|item| item.code == "MORVA2014"));
    assert!(!diagnostics.iter().any(|item| item.code == "MORVA2016"));
}

#[test]
fn decimal_targets_reject_integer_paths() {
    let source = r#"system Test {
  entity Account {
    balance: Decimal
    count: Integer
  }
  action Copy(account: Account) {
    effects account.balance = account.count
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "MORVA2016");
}

#[test]
fn entity_values_cannot_be_compared_as_whole_objects() {
    let source = r#"system Test {
  entity Item {}
  action Compare(first: Item, second: Item) {
    requires first == second
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert_eq!(
        diagnostics
            .iter()
            .filter(|item| item.code == "MORVA2014")
            .count(),
        1
    );
}

#[test]
fn resolution_failures_suppress_derived_type_diagnostics() {
    let source = r#"system Test {
  entity Item { count: Integer }
  action Broken(item: Item) {
    requires item.missing == 0
    effects item.count = missing
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert!(diagnostics.iter().any(|item| item.code == "MORVA2010"));
    assert!(diagnostics.iter().any(|item| item.code == "MORVA2009"));
    assert!(
        diagnostics
            .iter()
            .all(|item| !("MORVA2013"..="MORVA2017").contains(&item.code))
    );
}

#[test]
fn a_resolution_failure_keeps_its_primary_message_and_span() {
    let source = r#"system Test {
  entity Item { count: Integer }
  action Broken(item: Item) {
    requires item.missing == 0
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "MORVA2010");
    assert_eq!(
        diagnostics[0].message,
        "entity 'Item' has no field named 'missing'"
    );
    let start = source.find("missing").expect("missing field location");
    assert_eq!(diagnostics[0].span.start, start);
    assert_eq!(diagnostics[0].span.end, start + "missing".len());
}

#[test]
fn scenario_expects_must_be_boolean() {
    let source = r#"system Test {
  action Observe {}
  scenario Broken {
    run Observe()
    expect 1
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert_eq!(
        diagnostics
            .iter()
            .filter(|item| item.code == "MORVA2013")
            .count(),
        1
    );
}

#[test]
fn scenario_diagnostics_remain_in_source_order() {
    let source = r#"system Test {
  action Observe {}
  scenario Broken {
    expect 1
    run Observe()
    given missing.value = 1
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert!(diagnostics.len() >= 4, "diagnostics: {diagnostics:?}");
    assert!(
        diagnostics
            .windows(2)
            .all(|pair| pair[0].span.start <= pair[1].span.start),
        "diagnostics are not in source order: {diagnostics:?}"
    );
}

#[test]
fn rejects_an_always_false_action_predicate() {
    let source = r#"system Test {
  action Impossible {
    requires false
  }
}
"#;
    let document = parse(source).expect("valid syntax");
    let diagnostics = check(&document);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "MORVA2018");
    assert_eq!(diagnostics[0].message, "predicate is always false");
    let start = source.find("false").expect("predicate location");
    assert_eq!(diagnostics[0].span.start, start);
    assert_eq!(diagnostics[0].span.end, start + "false".len());
}

#[test]
fn rejects_constant_and_same_phase_literal_contradictions() {
    let source = r#"system Test {
  enum State {
    Ready
    Done
  }
  entity Item {
    state: State
    count: Integer
    enabled: Boolean
  }
  action Impossible(item: Item) {
    requires 1 != 1
    requires item.state == Ready
    invariant Done == item.state
    ensures item.enabled == true
    ensures false == item.enabled
  }
}
"#;
    let diagnostics = check(&parse(source).expect("valid syntax"));
    assert_eq!(
        diagnostics
            .iter()
            .map(|item| (item.code, item.message.as_str()))
            .collect::<Vec<_>>(),
        [
            ("MORVA2018", "predicate is always false"),
            (
                "MORVA2018",
                "predicate conflicts with an earlier literal constraint on 'item.state'"
            ),
            (
                "MORVA2018",
                "predicate conflicts with an earlier literal constraint on 'item.enabled'"
            ),
        ]
    );
    for (diagnostic, predicate) in
        diagnostics
            .iter()
            .zip(["1 != 1", "Done == item.state", "false == item.enabled"])
    {
        let start = source.find(predicate).expect("predicate location");
        assert_eq!(diagnostic.span.start, start);
        assert_eq!(diagnostic.span.end, start + predicate.len());
    }
}

#[test]
fn final_literal_effects_reject_conflicting_postconditions_conservatively() {
    let source = r#"system Test {
  enum State {
    Ready
    Done
  }
  entity Item {
    state: State
    count: Integer
    balance: Decimal
  }
  action Known(item: Item) {
    requires item.state == Ready
    effects item.state = Done
    ensures item.state != Done
  }
  action Overwrite(item: Item) {
    effects item.count = 1
    effects item.count = 2
    ensures item.count == 1
  }
  action Recovered(item: Item) {
    effects item.count = item.count
    effects item.count += 1
    effects item.count = 3
    ensures item.count == 4
  }
  action DecimalKnown(item: Item) {
    effects item.balance = 0
    ensures item.balance != 0
  }
  action Unknown(item: Item) {
    effects item.count = 3
    effects item.count += 1
    ensures item.count == 3
  }
}
"#;
    let diagnostics = check(&parse(source).expect("valid syntax"));
    assert_eq!(diagnostics.len(), 4, "diagnostics: {diagnostics:?}");
    for (diagnostic, predicate, path) in [
        (&diagnostics[0], "item.state != Done", "item.state"),
        (&diagnostics[1], "item.count == 1", "item.count"),
        (&diagnostics[2], "item.count == 4", "item.count"),
        (&diagnostics[3], "item.balance != 0", "item.balance"),
    ] {
        assert_eq!(diagnostic.code, "MORVA2019");
        assert_eq!(
            diagnostic.message,
            format!("postcondition conflicts with final literal effect for '{path}'")
        );
        let start = source.find(predicate).expect("predicate location");
        assert_eq!(diagnostic.span.start, start);
        assert_eq!(diagnostic.span.end, start + predicate.len());
    }
}

#[test]
fn literal_fact_analysis_preserves_legal_and_unknown_transitions() {
    let source = r#"system Test {
  enum State {
    Ready
    Done
  }
  entity Item {
    state: State
    other: State
    count: Integer
  }
  action Legal(item: Item) {
    requires item.state == Ready
    requires item.state == Ready
    requires item.other != Ready
    requires item.other != Done
    effects item.state = Done
    ensures item.state == Done
  }
  action UnknownSet(item: Item) {
    effects item.count = item.count
    ensures item.count == 1
  }
  action UnknownCompound(item: Item) {
    effects item.count = 1
    effects item.count += 1
    ensures item.count == 1
  }
}
"#;
    assert!(
        check(&parse(source).expect("valid syntax")).is_empty(),
        "legal transitions and unknown final values must not be inferred"
    );
}

#[test]
fn enum_member_facts_do_not_shadow_action_parameters() {
    let source = r#"system Test {
  enum State {
    Ready
    Done
  }
  entity Item { state: State }
  action ParameterValue(Ready: State, item: Item) {
    requires item.state == Ready
    requires item.state == Done
    effects item.state = Ready
    ensures item.state == Done
  }
}
"#;
    assert!(
        check(&parse(source).expect("valid syntax")).is_empty(),
        "a bound action parameter is not an enum literal fact"
    );
}

#[test]
fn primary_resolution_and_type_errors_suppress_literal_fact_diagnostics() {
    let source = r#"system Test {
  enum State { Ready }
  entity Item {
    count: Integer
    state: State
  }
  action Broken(item: Item) {
    requires item.missing == 0
    requires item.count == true
    requires item.state == Missing
    requires false
    effects item.count = 1
    ensures item.count == 2
  }
}
"#;
    let diagnostics = check(&parse(source).expect("valid syntax"));
    assert!(diagnostics.iter().any(|item| item.code == "MORVA2010"));
    assert!(diagnostics.iter().any(|item| item.code == "MORVA2014"));
    assert!(diagnostics.iter().any(|item| item.code == "MORVA2012"));
    assert!(
        diagnostics
            .iter()
            .all(|item| !matches!(item.code, "MORVA2018" | "MORVA2019"))
    );
}

#[test]
fn a_postcondition_gets_one_primary_contradiction_diagnostic_per_span() {
    let source = r#"system Test {
  entity Item { count: Integer }
  action Impossible(item: Item) {
    invariant item.count == 1
    effects item.count = 1
    ensures item.count == 2
  }
}
"#;
    let diagnostics = check(&parse(source).expect("valid syntax"));
    assert_eq!(diagnostics.len(), 1, "diagnostics: {diagnostics:?}");
    assert_eq!(diagnostics[0].code, "MORVA2018");
    assert_eq!(
        diagnostics[0].message,
        "predicate conflicts with an earlier literal constraint on 'item.count'"
    );
}

#[test]
fn literal_fact_diagnostics_remain_in_source_order_across_codes() {
    let source = r#"system Test {
  entity Item {
    first: Integer
    second: Integer
  }
  action Impossible(item: Item) {
    effects item.first = 1
    ensures item.first == 2
    ensures item.second != 3
    ensures item.second == 3
  }
}
"#;
    let diagnostics = check(&parse(source).expect("valid syntax"));
    assert_eq!(
        diagnostics.iter().map(|item| item.code).collect::<Vec<_>>(),
        ["MORVA2019", "MORVA2018"]
    );
    assert!(diagnostics[0].span.start < diagnostics[1].span.start);
    assert!(diagnostics[0].message.contains("item.first"));
    assert!(diagnostics[1].message.contains("item.second"));
}

#[test]
fn unknown_action_items_fail_but_documented_soft_items_remain_compatible() {
    let accepted = r#"system Shop {
  action Save {
    atomic
    idempotent by request.id
    timeout 10
    retry 2
    implementation_hint {
      storage: relational
    }
  }
}
"#;
    let document = parse(accepted).expect("documented soft items parse");
    assert!(check(&document).is_empty());

    let rejected = "system Shop {\n  action Save {\n    require true\n  }\n}\n";
    assert_eq!(
        parse(rejected).expect_err("typo must fail")[0].code,
        "MORVA1007"
    );
}

#[test]
fn a_container_missing_its_block_cannot_consume_the_next_declaration() {
    let source = "system Shop {\n  module Orders\n  entity Order {}\n}\n";
    let diagnostic = &parse(source).expect_err("missing block must fail")[0];
    assert_eq!(diagnostic.code, "MORVA1016");
    assert_eq!(
        diagnostic.span.start,
        source.find("entity").expect("entity keyword")
    );
}

#[test]
fn declaration_blocks_may_start_on_the_next_line() {
    let source = r#"system Shop
{
  enum State
  {
    Ready
  }
  entity Item
  {
    state: State
  }
  action Check(item: Item)
  {
    requires item.state == Ready
  }
}
"#;
    let document = parse(source).expect("next-line blocks remain compatible");
    assert!(check(&document).is_empty());
}

#[test]
fn clause_blocks_may_start_on_the_next_line() {
    let source = r#"system Shop {
  action Check {
    requires
    {
      true
    }
  }
}
"#;
    let document = parse(source).expect("next-line clause block parses");
    assert!(check(&document).is_empty());
}

#[test]
fn a_same_line_declaration_cannot_become_a_missing_container_block() {
    let source = "system Shop { module Orders entity Order {} }";
    assert_eq!(
        parse(source).expect_err("missing module block must fail")[0].code,
        "MORVA1016"
    );
}

#[test]
fn an_action_without_parentheses_is_compatible() {
    let document = parse("system Shop { action Refresh {} }").expect("action parses");
    assert!(check(&document).is_empty());
    let Declaration::System(system) = &document.declarations[0] else {
        panic!("expected system");
    };
    let Declaration::Action(action) = &system.declarations[0] else {
        panic!("expected action");
    };
    assert!(action.parameters.is_empty());
}

#[test]
fn booleans_and_other_keywords_cannot_be_names() {
    for source in [
        "system true {}",
        "system Shop { entity false {} }",
        "system Shop { entity Order { action: String } }",
    ] {
        assert_eq!(
            parse(source).expect_err("reserved name must fail")[0].code,
            "MORVA1019"
        );
    }
}

#[test]
fn scenario_item_keywords_are_contextual_names() {
    let source = r#"system Shop {
  enum State { given }
  entity Job { run: Boolean }
  action Check(expect: Job) {}
  scenario Case {
    given given.run = true
    run Check(given)
    expect given.run == true
  }
}
"#;
    let document = parse(source).expect("scenario item keywords remain contextual");
    assert!(check(&document).is_empty());
}

#[test]
fn out_of_range_integer_is_a_diagnostic_not_a_panic() {
    let source = "system Shop { action Check { requires 999999999999999999999999999 } }";
    assert_eq!(
        parse(source).expect_err("overflow must fail")[0].code,
        "MORVA1012"
    );
}

#[test]
fn nested_systems_are_rejected() {
    let source = "system Outer { module Inner { system Nested {} } }";
    assert!(codes(source).contains(&"MORVA2002"));
}

#[test]
fn exactly_one_system_must_be_at_the_document_root() {
    assert!(codes("entity Loose {}").contains(&"MORVA2001"));
    assert!(codes("system One {} system Two {}").contains(&"MORVA2001"));
}

#[test]
fn globally_ambiguous_short_type_names_are_rejected() {
    let first = r#"system Shop {
  module A { entity Item {} }
  module B { entity Item {} }
}
"#;
    let second = r#"system Shop {
  module B { entity Item {} }
  module A { entity Item {} }
}
"#;
    assert!(codes(first).contains(&"MORVA2008"));
    assert!(codes(second).contains(&"MORVA2008"));
}

#[test]
fn user_types_cannot_shadow_builtin_types() {
    assert!(codes("system Shop { entity String {} }").contains(&"MORVA2008"));
}

#[test]
fn non_ascii_diagnostics_cover_the_complete_codepoint() {
    let diagnostic = &parse("system Café {}").expect_err("non-ASCII name must fail")[0];
    assert_eq!(diagnostic.span.end - diagnostic.span.start, 'é'.len_utf8());
}

#[test]
fn incomplete_syntax_has_a_span() {
    let diagnostic =
        &parse("system Shop { entity Order { id ID } }").expect_err("missing colon must fail")[0];
    assert_eq!(diagnostic.code, "MORVA1006");
    assert!(diagnostic.span.end > diagnostic.span.start);
}

#[test]
fn soft_behavior_keyword_spans_preserve_all_newline_byte_sequences() {
    for newline in ["\n", "\r\n", "\r"] {
        let source = [
            "system Shop {",
            newline,
            "  action Save {",
            newline,
            "    timeout 30",
            newline,
            "    implementation_hint",
            newline,
            "    { nested { value } }",
            newline,
            "  }",
            newline,
            "}",
            newline,
        ]
        .concat();
        let document = parse(&source).expect("newline variant parses");
        let report = analyze(&document);
        assert_eq!(report.notices.len(), 2);
        for (notice, keyword) in report
            .notices
            .iter()
            .zip(["timeout", "implementation_hint"])
        {
            let start = source.find(keyword).unwrap();
            assert_eq!(notice.span.start, start, "newline {newline:?}");
            assert_eq!(
                notice.span.end,
                start + keyword.len(),
                "newline {newline:?}"
            );
        }
    }
}

#[test]
fn malformed_soft_behaviors_keep_existing_parser_errors_without_partial_analysis() {
    for (source, code, message, marker, span_len) in [
        (
            "system Shop {\n  action Save {\n    typo\n  }\n}\n",
            "MORVA1007",
            "unknown item in action block",
            "typo",
            4,
        ),
        (
            "system Shop {\n  action Save {\n    atomic { value }\n  }\n}\n",
            "MORVA1014",
            "unexpected block for soft behavior item",
            "{ value }",
            1,
        ),
        (
            "system Shop {\n  action Save {\n    implementation_hint\n  }\n}\n",
            "MORVA1016",
            "expected block after implementation_hint",
            "}",
            1,
        ),
        (
            "system Shop {\n  action Save {\n    implementation_hint { nested\n",
            "MORVA1003",
            "unclosed compatibility block",
            "{ nested",
            1,
        ),
    ] {
        let diagnostics = parse(source).expect_err("malformed input must not produce an AST");
        assert_eq!(diagnostics.len(), 1, "{source:?}");
        assert_eq!(diagnostics[0].code, code, "{source:?}");
        assert_eq!(diagnostics[0].message, message, "{source:?}");
        assert_eq!(
            diagnostics[0].span.start,
            source.find(marker).unwrap(),
            "{source:?}"
        );
        assert_eq!(
            diagnostics[0].span.end - diagnostics[0].span.start,
            span_len,
            "{source:?}"
        );
    }
}

#[test]
fn soft_container_and_semantic_findings_share_source_order_and_legacy_check_parity() {
    let source = r#"system Shop {
  module Compat {}
  action Save {
    retry 2
    retry 3
  }
  entity Item { value: Missing }
}
"#;
    let document = parse(source).expect("syntax valid");
    let report = analyze(&document);
    assert_eq!(report.errors, check(&document));
    assert_eq!(report.notices.len(), 3);
    assert!(matches!(
        report.findings().as_slice(),
        [
            AnalysisFinding::Notice(_),
            AnalysisFinding::Notice(_),
            AnalysisFinding::Notice(_),
            AnalysisFinding::Error(_)
        ]
    ));
    assert_eq!(
        report
            .notices
            .iter()
            .map(|notice| notice.code)
            .collect::<Vec<_>>(),
        ["MORVA5001", "MORVA5002", "MORVA5002"]
    );
}
