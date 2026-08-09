use morva_core::{ClauseExpression, ClauseKind, Declaration, check, parse};

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
fn parses_the_complete_strongly_typed_core() {
    let document = parse(COMPLETE_MODEL).expect("valid model");
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
