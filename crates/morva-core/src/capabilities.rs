use crate::ast::{AssignmentOperator, BinaryOperator, ClauseKind, SoftBehaviorKind};
use crate::parser::{COMPATIBILITY_CONTAINER_KINDS, SEMANTIC_DECLARATION_KINDS};
use crate::semantic::{BUILTIN_TYPE_ALIASES, BUILTIN_TYPE_NAMES};
use crate::simulate::SimulationPhase;

/// Bumped only on reviewed contract changes to the inventory shape or content.
pub const CAPABILITY_INVENTORY_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityInventory {
    pub version: u32,
    pub declarations: Vec<&'static str>,
    pub clause_kinds: Vec<&'static str>,
    pub expression_forms: Vec<&'static str>,
    pub comparison_operators: Vec<&'static str>,
    pub assignment_operators: Vec<&'static str>,
    pub literals: Vec<&'static str>,
    pub builtin_types: Vec<&'static str>,
    pub builtin_type_aliases: Vec<(&'static str, &'static str)>,
    pub simulation_value_types: Vec<&'static str>,
    pub simulation_phases: Vec<&'static str>,
    pub compatibility_containers: Vec<&'static str>,
    pub soft_behaviors: Vec<&'static str>,
    pub unsupported: Vec<&'static str>,
}

pub fn capabilities() -> CapabilityInventory {
    CapabilityInventory {
        version: CAPABILITY_INVENTORY_VERSION,
        declarations: SEMANTIC_DECLARATION_KINDS.to_vec(),
        clause_kinds: ClauseKind::ALL.map(ClauseKind::as_str).to_vec(),
        expression_forms: vec![
            "path reference",
            "Integer literal",
            "Boolean literal",
            "binary comparison",
        ],
        comparison_operators: BinaryOperator::ALL.map(BinaryOperator::as_str).to_vec(),
        assignment_operators: AssignmentOperator::ALL
            .map(AssignmentOperator::as_str)
            .to_vec(),
        literals: vec!["Integer", "Boolean", "enum member"],
        builtin_types: BUILTIN_TYPE_NAMES.to_vec(),
        builtin_type_aliases: BUILTIN_TYPE_ALIASES.to_vec(),
        simulation_value_types: vec!["enum member", "Boolean", "Integer"],
        simulation_phases: SimulationPhase::ALL.map(SimulationPhase::as_str).to_vec(),
        compatibility_containers: COMPATIBILITY_CONTAINER_KINDS.to_vec(),
        soft_behaviors: SoftBehaviorKind::ALL.map(SoftBehaviorKind::as_str).to_vec(),
        unsupported: vec![
            "logical 'or', 'not', and grouped predicates",
            "arithmetic expressions",
            "String literals",
            "Decimal literals",
            "negative number literals",
            "collections and one-to-many relationships",
            "optional fields",
            "quantifiers",
            "date, time, and duration types",
            "multi-run scenarios",
            "module scoping and qualified names",
            "state machine semantics for lifecycle",
            "machine-readable output formats",
        ],
    }
}
