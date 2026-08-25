use morva_core::protocol::{
    ByteRange, CheckedSemanticsDocument, CoverageAssessment, CoverageDetails, DeclarationNode,
    ExpressionNode, FindingCategory, ProtocolBuildError, ProtocolInvariantError, ProtocolLocation,
    ResultStatus, Severity, checked_semantics_single_file,
};

const CHECKED_SOURCE: &str = r#"system Shop {
  enum Status {
    Pending
    Confirmed
  }
  entity Order {
    status: Status
    total: Decimal
    vip: Boolean
    invariant total >= 0
  }
  action Confirm(order: Order) {
    requires order.vip || !(order.status == Confirmed)
    effects order.status = Confirmed
    ensures order.status == Confirmed
  }
  scenario Normal {
    given order.status = Pending
    given order.vip = false
    run Confirm(order)
    expect order.status == Confirmed
  }
}
"#;

fn document(source: &str) -> CheckedSemanticsDocument {
    checked_semantics_single_file("model.morva", source).expect("protocol production succeeds")
}

#[test]
fn a_checked_source_produces_a_valid_complete_document() {
    let document = document(CHECKED_SOURCE);
    assert_eq!(document.subject.name, "model.morva");
    assert_eq!(document.sources.len(), 1);
    assert_eq!(document.sources[0].source_id, "source:0");
    assert_eq!(document.sources[0].content, CHECKED_SOURCE);
    assert_eq!(document.sources[0].revision, document.subject.revision);
    assert_eq!(document.result.status, ResultStatus::Valid);
    assert!(document.result.findings.is_empty());
    assert_eq!(
        document.result.coverage.assessment,
        CoverageAssessment::Complete
    );
    assert_eq!(document.result.coverage.fully_modeled, Some(true));
    let model = document.result.checked_model.as_ref().expect("model");
    assert_eq!(model.system.name, "Shop");
    assert_eq!(model.system.semantic_key, "system:Shop");
    assert_eq!(model.system.declarations.len(), 4);
    let DeclarationNode::Action(action) = &model.system.declarations[2] else {
        panic!("third declaration is the action");
    };
    assert_eq!(action.semantic_key, "action:Confirm");
    assert_eq!(action.parameters[0].semantic_key, "parameter:Confirm.order");
    assert_eq!(action.clauses.len(), 3);
    assert_eq!(action.clauses[0].clause_kind, "requires");
    assert_eq!(action.clauses[0].state_phase, "pre");
    assert_eq!(action.clauses[1].clause_kind, "effects");
    assert_eq!(action.clauses[1].state_phase, "effect");
    document.validate().expect("document validates");
}

#[test]
fn semantic_errors_produce_an_invalid_document_without_a_model() {
    let document = document("system Shop {\n  entity Order { status: Missing }\n}\n");
    assert_eq!(document.result.status, ResultStatus::Invalid);
    assert!(document.result.checked_model.is_none());
    assert_eq!(document.result.findings.len(), 1);
    let finding = &document.result.findings[0];
    assert_eq!(finding.finding_id, "finding:0");
    assert_eq!(finding.severity, Severity::Error);
    assert_eq!(finding.category, FindingCategory::Semantic);
    assert_eq!(finding.code, "MORVA2007");
    assert_eq!(
        document.result.coverage.assessment,
        CoverageAssessment::Complete
    );
    assert_eq!(document.result.coverage.fully_modeled, Some(true));
    document
        .validate()
        .expect("invalid documents still validate");
}

#[test]
fn lexical_and_syntax_failures_use_typed_pipeline_categories() {
    let lexical = document("system Shop {\n  \u{1}\n}\n");
    assert_eq!(
        lexical.result.findings[0].category,
        FindingCategory::Lexical
    );
    assert_eq!(lexical.result.findings[0].code, "MORVA1001");
    assert_eq!(
        lexical.result.coverage.assessment,
        CoverageAssessment::Unavailable
    );
    assert_eq!(lexical.result.coverage.fully_modeled, None);
    lexical.validate().expect("lexical failure validates");

    let syntax = document("system Shop {\n  action Broken(order Order) {}\n}\n");
    assert_eq!(syntax.result.findings[0].category, FindingCategory::Syntax);
    assert_eq!(syntax.result.findings[0].code, "MORVA1008");
    assert!(syntax.result.checked_model.is_none());
    syntax.validate().expect("syntax failure validates");
}

#[test]
fn coverage_warnings_map_one_to_one_into_unmodeled_entries() {
    let document =
        document("system Shop {\n  module Compat {}\n  action Save {\n    retry 2\n  }\n}\n");
    assert_eq!(document.result.status, ResultStatus::Valid);
    assert_eq!(document.result.coverage.fully_modeled, Some(false));
    assert_eq!(document.result.coverage.unmodeled.len(), 2);
    assert_eq!(document.result.findings.len(), 2);
    for finding in &document.result.findings {
        assert_eq!(finding.severity, Severity::Warning);
        assert_eq!(finding.category, FindingCategory::Coverage);
    }
    assert_eq!(
        document.result.coverage.unmodeled[0].details,
        CoverageDetails::CompatibilityContainer {
            container_kind: "module".to_owned(),
            name: "Compat".to_owned(),
        }
    );
    assert_eq!(
        document.result.coverage.unmodeled[1].details,
        CoverageDetails::ActionSoftBehavior {
            action: "Save".to_owned(),
            behavior: "retry",
        }
    );
    let model = document.result.checked_model.as_ref().expect("model");
    let DeclarationNode::Action(action) = &model.system.declarations[0] else {
        panic!("action declaration");
    };
    assert!(
        action.clauses.is_empty(),
        "soft behavior stays out of the model"
    );
    document
        .validate()
        .expect("warning-only document validates");
}

#[test]
fn logical_names_reject_separators_and_empty_input() {
    for name in ["", "a/b.morva", "a\\b.morva"] {
        assert!(matches!(
            checked_semantics_single_file(name, "system Shop {}\n"),
            Err(ProtocolBuildError::InvalidLogicalName(_))
        ));
    }
}

#[test]
fn canonical_json_matches_the_independently_checked_literal() {
    let document = document("system Shop {}\n");
    let first = document.to_canonical_json().expect("serializes");
    let second = document.to_canonical_json().expect("serializes again");
    assert_eq!(first, second, "repeated serialization is byte-identical");
    let expected = r#"{
  "protocol": "morva.checked-semantics",
  "version": 1,
  "capabilities": [
    "morva.sources.inline",
    "morva.locations.byte-range",
    "morva.findings.v1",
    "morva.coverage.v1",
    "morva.checked-model.v1"
  ],
  "producer": {
    "name": "morva",
    "version": "0.1.0"
  },
  "subject": {
    "kind": "file",
    "name": "model.morva",
    "revision": {
      "algorithm": "sha256",
      "value": "2d9cae25046e1bf3b847d4aaf00156f2c6dd2d63d7a413421b9d6ab8060c0047"
    }
  },
  "sources": [
    {
      "source_id": "source:0",
      "name": "model.morva",
      "content_encoding": "utf-8",
      "content": "system Shop {}\n",
      "revision": {
        "algorithm": "sha256",
        "value": "2d9cae25046e1bf3b847d4aaf00156f2c6dd2d63d7a413421b9d6ab8060c0047"
      }
    }
  ],
  "result": {
    "status": "valid",
    "findings": [],
    "coverage": {
      "assessment": "complete",
      "fully_modeled": true,
      "unmodeled": []
    },
    "checked_model": {
      "language": "morva",
      "language_version": "0.1",
      "system": {
        "kind": "system",
        "node_id": "node:0",
        "semantic_key": "system:Shop",
        "name": "Shop",
        "shell_locations": [
          {
            "kind": "source",
            "source_id": "source:0",
            "byte_range": {
              "start": 0,
              "end": 14
            }
          }
        ],
        "declarations": []
      }
    }
  }
}
"#;
    assert_eq!(first, expected);
}

#[test]
fn the_full_document_round_trips_and_repeats_byte_identically() {
    let document = document(CHECKED_SOURCE);
    let first = document.to_canonical_json().expect("serializes");
    let again = self::document(CHECKED_SOURCE)
        .to_canonical_json()
        .expect("serializes");
    assert_eq!(first, again, "same input produces identical bytes");
    assert!(first.ends_with('\n'));
    assert!(first.contains("\"kind\": \"or\""));
    assert!(first.contains("\"kind\": \"not\""));
    assert!(first.contains("\"operator\": \"greater_equal\""));

    // The Decimal comparison context types the 0 literal as Decimal.
    let model = document.result.checked_model.as_ref().expect("model");
    let DeclarationNode::Entity(entity) = &model.system.declarations[1] else {
        panic!("entity declaration");
    };
    let ExpressionNode::Binary { right, .. } = &entity.invariants[0] else {
        panic!("invariant comparison");
    };
    let ExpressionNode::Integer {
        value,
        resolved_type,
        ..
    } = right.as_ref()
    else {
        panic!("integer literal");
    };
    assert_eq!(value, "0");
    assert_eq!(
        resolved_type,
        &morva_core::protocol::TypeRef::Builtin { name: "Decimal" }
    );
}

#[test]
fn the_validator_rejects_each_mutated_invariant() {
    let base = document(CHECKED_SOURCE);

    let mut wrong_digest = base.clone();
    wrong_digest.sources[0].revision.value = "0".repeat(64);
    assert!(matches!(
        wrong_digest.validate(),
        Err(ProtocolInvariantError::Source(_))
    ));

    let mut subject_drift = base.clone();
    subject_drift.subject.revision.value = "0".repeat(64);
    assert!(matches!(
        subject_drift.validate(),
        Err(ProtocolInvariantError::Source(_))
    ));

    let mut bad_range = base.clone();
    if let Some(model) = &mut bad_range.result.checked_model {
        model.system.shell_locations = vec![ProtocolLocation::Source {
            source_id: "source:0".to_owned(),
            byte_range: ByteRange {
                start: 0,
                end: 1_000_000,
            },
        }];
    }
    assert!(matches!(
        bad_range.validate(),
        Err(ProtocolInvariantError::Location(_))
    ));

    let mut status_mismatch = base.clone();
    status_mismatch.result.status = ResultStatus::Invalid;
    assert!(matches!(
        status_mismatch.validate(),
        Err(ProtocolInvariantError::Status(_))
    ));

    let mut duplicate_nodes = base.clone();
    if let Some(model) = &mut duplicate_nodes.result.checked_model
        && let DeclarationNode::Enum(enumeration) = &mut model.system.declarations[0]
    {
        enumeration.node_id = model.system.node_id.clone();
    }
    assert!(matches!(
        duplicate_nodes.validate(),
        Err(ProtocolInvariantError::Identity(_))
    ));

    let mut dangling_reference = base.clone();
    if let Some(model) = &mut dangling_reference.result.checked_model
        && let DeclarationNode::Entity(entity) = &mut model.system.declarations[1]
    {
        entity.fields[0].field_type = morva_core::protocol::TypeRef::Enum {
            semantic_key: "enum:Missing".to_owned(),
        };
    }
    assert!(matches!(
        dangling_reference.validate(),
        Err(ProtocolInvariantError::Reference(_))
    ));

    let mut wrong_phase = base.clone();
    if let Some(model) = &mut wrong_phase.result.checked_model
        && let DeclarationNode::Action(action) = &mut model.system.declarations[2]
    {
        action.clauses[0].state_phase = "post";
    }
    assert!(matches!(
        wrong_phase.validate(),
        Err(ProtocolInvariantError::Model(_))
    ));

    let mut missing_declaration = base.clone();
    if let Some(model) = &mut missing_declaration.result.checked_model {
        model.system.declarations.pop();
    }
    assert!(matches!(
        missing_declaration.validate(),
        Err(ProtocolInvariantError::Model(_))
    ));

    let mut coverage_drift = base.clone();
    coverage_drift.result.coverage.fully_modeled = Some(false);
    assert!(matches!(
        coverage_drift.validate(),
        Err(ProtocolInvariantError::Coverage(_))
    ));

    assert!(
        base.validate().is_ok(),
        "the unmutated document stays valid"
    );
}

#[test]
fn serialization_refuses_a_document_that_fails_validation() {
    let mut document = document(CHECKED_SOURCE);
    document.sources[0].revision.value = "0".repeat(64);
    assert!(document.to_canonical_json().is_err());
}
