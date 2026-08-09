#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub(crate) fn covering(start: Self, end: Self) -> Self {
        Self {
            start: start.start,
            end: end.end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub declarations: Vec<Declaration>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    System(System),
    Entity(Entity),
    Enum(Enum),
    Action(Action),
    Scenario(Scenario),
    Container(Container),
}

impl Declaration {
    pub fn kind(&self) -> &str {
        match self {
            Self::System(_) => "system",
            Self::Entity(_) => "entity",
            Self::Enum(_) => "enum",
            Self::Action(_) => "action",
            Self::Scenario(_) => "scenario",
            Self::Container(item) => &item.kind,
        }
    }

    pub fn name(&self) -> &Name {
        match self {
            Self::System(item) => &item.name,
            Self::Entity(item) => &item.name,
            Self::Enum(item) => &item.name,
            Self::Action(item) => &item.name,
            Self::Scenario(item) => &item.name,
            Self::Container(item) => &item.name,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::System(item) => item.span,
            Self::Entity(item) => item.span,
            Self::Enum(item) => item.span,
            Self::Action(item) => item.span,
            Self::Scenario(item) => item.span,
            Self::Container(item) => item.span,
        }
    }

    pub fn declarations(&self) -> &[Declaration] {
        match self {
            Self::System(item) => &item.declarations,
            Self::Container(item) => &item.declarations,
            Self::Entity(_) | Self::Enum(_) | Self::Action(_) | Self::Scenario(_) => &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct System {
    pub name: Name,
    pub declarations: Vec<Declaration>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub kind: String,
    pub name: Name,
    pub declarations: Vec<Declaration>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub name: Name,
    pub fields: Vec<Field>,
    pub invariants: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enum {
    pub name: Name,
    pub members: Vec<Name>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: Name,
    pub type_name: Name,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub name: Name,
    pub parameters: Vec<Parameter>,
    pub clauses: Vec<Clause>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: Name,
    pub type_name: Name,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub name: Name,
    pub items: Vec<ScenarioItem>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioItem {
    Given(Assignment),
    Run(Run),
    Expect(Expr),
}

impl ScenarioItem {
    pub fn span(&self) -> Span {
        match self {
            Self::Given(item) => item.span,
            Self::Run(item) => item.span,
            Self::Expect(item) => item.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub action: Name,
    pub arguments: Vec<Name>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    pub kind: ClauseKind,
    pub expressions: Vec<ClauseExpression>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClauseKind {
    Requires,
    Effects,
    Ensures,
    Invariant,
}

impl ClauseKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Requires => "requires",
            Self::Effects => "effects",
            Self::Ensures => "ensures",
            Self::Invariant => "invariant",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClauseExpression {
    Predicate(Expr),
    Assignment(Assignment),
}

impl ClauseExpression {
    pub fn span(&self) -> Span {
        match self {
            Self::Predicate(item) => item.span,
            Self::Assignment(item) => item.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub target: Path,
    pub operator: AssignmentOperator,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOperator {
    Set,
    Add,
    Subtract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Integer(i64),
    Boolean(bool),
    Path(Path),
    Binary {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub segments: Vec<Name>,
    pub span: Span,
}

impl Path {
    pub fn display(&self) -> String {
        self.segments
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }
}
