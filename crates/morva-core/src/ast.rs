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

    pub(crate) fn checked_rebase(self, base: usize) -> Option<Self> {
        Some(Self {
            start: self.start.checked_add(base)?,
            end: self.end.checked_add(base)?,
        })
    }
}

pub(crate) trait RebaseSpans {
    fn rebase_spans(&mut self, base: usize) -> Option<()>;
}

impl RebaseSpans for Span {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        *self = self.checked_rebase(base)?;
        Some(())
    }
}

impl RebaseSpans for Name {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.span.rebase_spans(base)
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

impl RebaseSpans for Document {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.span.rebase_spans(base)?;
        for declaration in &mut self.declarations {
            declaration.rebase_spans(base)?;
        }
        Some(())
    }
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

impl RebaseSpans for Declaration {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        match self {
            Self::System(item) => item.rebase_spans(base),
            Self::Entity(item) => item.rebase_spans(base),
            Self::Enum(item) => item.rebase_spans(base),
            Self::Action(item) => item.rebase_spans(base),
            Self::Scenario(item) => item.rebase_spans(base),
            Self::Container(item) => item.rebase_spans(base),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct System {
    pub name: Name,
    pub declarations: Vec<Declaration>,
    pub span: Span,
}

macro_rules! rebase_named_declarations {
    ($type:ty) => {
        impl RebaseSpans for $type {
            fn rebase_spans(&mut self, base: usize) -> Option<()> {
                self.name.rebase_spans(base)?;
                self.span.rebase_spans(base)?;
                for declaration in &mut self.declarations {
                    declaration.rebase_spans(base)?;
                }
                Some(())
            }
        }
    };
}

rebase_named_declarations!(System);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub kind: String,
    pub name: Name,
    pub declarations: Vec<Declaration>,
    pub span: Span,
}

rebase_named_declarations!(Container);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub name: Name,
    pub fields: Vec<Field>,
    pub invariants: Vec<Expr>,
    pub span: Span,
}

impl RebaseSpans for Entity {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.name.rebase_spans(base)?;
        self.span.rebase_spans(base)?;
        for field in &mut self.fields {
            field.rebase_spans(base)?;
        }
        for invariant in &mut self.invariants {
            invariant.rebase_spans(base)?;
        }
        Some(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enum {
    pub name: Name,
    pub members: Vec<Name>,
    pub span: Span,
}

impl RebaseSpans for Enum {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.name.rebase_spans(base)?;
        self.span.rebase_spans(base)?;
        for member in &mut self.members {
            member.rebase_spans(base)?;
        }
        Some(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: Name,
    pub type_name: Name,
    pub span: Span,
}

impl RebaseSpans for Field {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.name.rebase_spans(base)?;
        self.type_name.rebase_spans(base)?;
        self.span.rebase_spans(base)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub name: Name,
    pub parameters: Vec<Parameter>,
    pub clauses: Vec<Clause>,
    pub soft_behaviors: Vec<SoftBehavior>,
    pub span: Span,
}

impl RebaseSpans for Action {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.name.rebase_spans(base)?;
        self.span.rebase_spans(base)?;
        for parameter in &mut self.parameters {
            parameter.rebase_spans(base)?;
        }
        for clause in &mut self.clauses {
            clause.rebase_spans(base)?;
        }
        for behavior in &mut self.soft_behaviors {
            behavior.rebase_spans(base)?;
        }
        Some(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftBehaviorKind {
    Atomic,
    Idempotent,
    Timeout,
    Retry,
    ImplementationHint,
}

impl SoftBehaviorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Atomic => "atomic",
            Self::Idempotent => "idempotent",
            Self::Timeout => "timeout",
            Self::Retry => "retry",
            Self::ImplementationHint => "implementation_hint",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftBehavior {
    pub kind: SoftBehaviorKind,
    pub span: Span,
}

impl RebaseSpans for SoftBehavior {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.span.rebase_spans(base)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: Name,
    pub type_name: Name,
    pub span: Span,
}

impl RebaseSpans for Parameter {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.name.rebase_spans(base)?;
        self.type_name.rebase_spans(base)?;
        self.span.rebase_spans(base)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub name: Name,
    pub items: Vec<ScenarioItem>,
    pub span: Span,
}

impl RebaseSpans for Scenario {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.name.rebase_spans(base)?;
        self.span.rebase_spans(base)?;
        for item in &mut self.items {
            item.rebase_spans(base)?;
        }
        Some(())
    }
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

impl RebaseSpans for ScenarioItem {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        match self {
            Self::Given(item) => item.rebase_spans(base),
            Self::Run(item) => item.rebase_spans(base),
            Self::Expect(item) => item.rebase_spans(base),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub action: Name,
    pub arguments: Vec<Name>,
    pub span: Span,
}

impl RebaseSpans for Run {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.action.rebase_spans(base)?;
        self.span.rebase_spans(base)?;
        for argument in &mut self.arguments {
            argument.rebase_spans(base)?;
        }
        Some(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    pub kind: ClauseKind,
    pub expressions: Vec<ClauseExpression>,
    pub span: Span,
}

impl RebaseSpans for Clause {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.span.rebase_spans(base)?;
        for expression in &mut self.expressions {
            expression.rebase_spans(base)?;
        }
        Some(())
    }
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

impl RebaseSpans for ClauseExpression {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        match self {
            Self::Predicate(item) => item.rebase_spans(base),
            Self::Assignment(item) => item.rebase_spans(base),
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

impl RebaseSpans for Assignment {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.target.rebase_spans(base)?;
        self.value.rebase_spans(base)?;
        self.span.rebase_spans(base)
    }
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

impl RebaseSpans for Expr {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.span.rebase_spans(base)?;
        match &mut self.kind {
            ExprKind::Path(path) => path.rebase_spans(base),
            ExprKind::Binary { left, right, .. } => {
                left.rebase_spans(base)?;
                right.rebase_spans(base)
            }
            ExprKind::Integer(_) | ExprKind::Boolean(_) => Some(()),
        }
    }
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

impl RebaseSpans for Path {
    fn rebase_spans(&mut self, base: usize) -> Option<()> {
        self.span.rebase_spans(base)?;
        for segment in &mut self.segments {
            segment.rebase_spans(base)?;
        }
        Some(())
    }
}
