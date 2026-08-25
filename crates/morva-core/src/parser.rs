use crate::Diagnostic;
use crate::ast::*;
use crate::lexer::{Token, TokenKind, lex};

pub(crate) const SEMANTIC_DECLARATION_KINDS: &[&str] =
    &["system", "entity", "enum", "action", "scenario"];

pub(crate) const COMPATIBILITY_CONTAINER_KINDS: &[&str] =
    &["module", "service", "event", "flow", "lifecycle", "policy"];

const RESERVED_NAMES: &[&str] = &[
    "system",
    "module",
    "entity",
    "enum",
    "service",
    "action",
    "event",
    "flow",
    "lifecycle",
    "scenario",
    "policy",
    "requires",
    "effects",
    "ensures",
    "invariant",
    "atomic",
    "idempotent",
    "timeout",
    "retry",
    "implementation_hint",
    "true",
    "false",
];

pub(crate) fn parse(source: &str) -> Result<Document, Vec<Diagnostic>> {
    let tokens = lex(source)?;
    Parser {
        tokens,
        cursor: 0,
        source_len: source.len(),
    }
    .document()
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    source_len: usize,
}

impl Parser {
    fn document(mut self) -> Result<Document, Vec<Diagnostic>> {
        let declarations = self.declarations(false)?;
        Ok(Document {
            declarations,
            span: Span {
                start: 0,
                end: self.source_len,
            },
        })
    }

    fn declarations(&mut self, closes: bool) -> Result<Vec<Declaration>, Vec<Diagnostic>> {
        let mut declarations = Vec::new();
        loop {
            self.newlines();
            if self.at_eof() {
                return if closes {
                    Err(self.error("MORVA1003", "unclosed declaration block"))
                } else {
                    Ok(declarations)
                };
            }
            if self.at_symbol('}') {
                if closes {
                    self.bump();
                    return Ok(declarations);
                }
                return Err(self.error("MORVA1004", "unexpected closing brace"));
            }
            if self.current_declaration_kind().is_some() {
                declarations.push(self.declaration()?);
            } else {
                return Err(self.error("MORVA1005", "expected a declaration"));
            }
        }
    }

    fn declaration(&mut self) -> Result<Declaration, Vec<Diagnostic>> {
        let kind_token = self.bump().clone();
        let kind = identifier_text(&kind_token)
            .expect("declaration keyword")
            .to_owned();
        let name = self.declared_name(&format!("missing name after {kind}"))?;
        match kind.as_str() {
            "system" => self.system(kind_token.span, name),
            "entity" => self.entity(kind_token.span, name),
            "enum" => self.enumeration(kind_token.span, name),
            "action" => self.action(kind_token.span, name),
            "scenario" => self.scenario(kind_token.span, name),
            _ => self.container(kind_token.span, kind, name),
        }
    }

    fn system(&mut self, start: Span, name: Name) -> Result<Declaration, Vec<Diagnostic>> {
        self.expect_block("expected '{' after system name")?;
        let declarations = self.declarations(true)?;
        Ok(Declaration::System(System {
            name,
            declarations,
            span: Span {
                start: start.start,
                end: self.previous().span.end,
            },
        }))
    }

    fn container(
        &mut self,
        start: Span,
        kind: String,
        name: Name,
    ) -> Result<Declaration, Vec<Diagnostic>> {
        self.skip_signature()?;
        self.expect_block("expected declaration block")?;
        let mut declarations = Vec::new();
        loop {
            self.newlines();
            if self.at_eof() {
                return Err(self.error("MORVA1003", "unclosed declaration block"));
            }
            if self.at_symbol('}') {
                self.bump();
                break;
            }
            if self.current_declaration_kind().is_some() {
                declarations.push(self.declaration()?);
            } else {
                self.skip_compatibility_item()?;
            }
        }
        Ok(Declaration::Container(Container {
            kind,
            name,
            declarations,
            span: Span {
                start: start.start,
                end: self.previous().span.end,
            },
        }))
    }

    fn enumeration(&mut self, start: Span, name: Name) -> Result<Declaration, Vec<Diagnostic>> {
        self.expect_block("expected '{' after enum name")?;
        let mut members = Vec::new();
        loop {
            self.newlines();
            if self.at_eof() {
                return Err(self.error("MORVA1003", "unclosed enum block"));
            }
            if self.at_symbol('}') {
                self.bump();
                break;
            }
            let member = self.declared_name("expected an enum member")?;
            members.push(member);
            if self.at_symbol(',') {
                self.bump();
            }
            self.line_end("unexpected token after enum member")?;
        }
        Ok(Declaration::Enum(Enum {
            name,
            members,
            span: Span {
                start: start.start,
                end: self.previous().span.end,
            },
        }))
    }

    fn entity(&mut self, start: Span, name: Name) -> Result<Declaration, Vec<Diagnostic>> {
        self.expect_block("expected '{' after entity name")?;
        let mut fields = Vec::new();
        let mut invariants = Vec::new();
        loop {
            self.newlines();
            if self.at_eof() {
                return Err(self.error("MORVA1003", "unclosed entity block"));
            }
            if self.at_symbol('}') {
                self.bump();
                break;
            }
            if self.at_identifier("invariant") {
                let clause = self.clause(ClauseKind::Invariant)?;
                invariants.extend(clause.expressions.into_iter().map(|item| match item {
                    ClauseExpression::Predicate(expr) => expr,
                    ClauseExpression::Assignment(_) => unreachable!("invariant predicate"),
                }));
            } else {
                fields.push(self.field()?);
            }
        }
        Ok(Declaration::Entity(Entity {
            name,
            fields,
            invariants,
            span: Span {
                start: start.start,
                end: self.previous().span.end,
            },
        }))
    }

    fn field(&mut self) -> Result<Field, Vec<Diagnostic>> {
        let name = self.declared_name("expected a field name")?;
        self.expect_symbol(':', "MORVA1006", "expected ':' after field name")?;
        let type_name = self.type_name("expected a field type")?;
        let span = Span::covering(name.span, type_name.span);
        self.line_end("unexpected token after field type")?;
        Ok(Field {
            name,
            type_name,
            span,
        })
    }

    fn action(&mut self, start: Span, name: Name) -> Result<Declaration, Vec<Diagnostic>> {
        let parameters = if self.at_symbol('(') {
            self.bump();
            self.parameters()?
        } else {
            Vec::new()
        };
        self.expect_block("expected action block")?;
        let mut clauses = Vec::new();
        let mut soft_behaviors = Vec::new();
        loop {
            self.newlines();
            if self.at_eof() {
                return Err(self.error("MORVA1003", "unclosed action block"));
            }
            if self.at_symbol('}') {
                self.bump();
                break;
            }
            let clause_kind = if self.at_identifier("requires") {
                Some(ClauseKind::Requires)
            } else if self.at_identifier("effects") {
                Some(ClauseKind::Effects)
            } else if self.at_identifier("ensures") {
                Some(ClauseKind::Ensures)
            } else if self.at_identifier("invariant") {
                Some(ClauseKind::Invariant)
            } else {
                None
            };
            if let Some(kind) = clause_kind {
                clauses.push(self.clause(kind)?);
            } else if self.at_identifier("implementation_hint") {
                soft_behaviors
                    .push(self.skip_implementation_hint(SoftBehaviorKind::ImplementationHint)?);
            } else if let Some(kind) = self.current_soft_behavior_kind() {
                soft_behaviors.push(self.skip_soft_behavior_line(kind)?);
            } else {
                return Err(self.error("MORVA1007", "unknown item in action block"));
            }
        }
        Ok(Declaration::Action(Action {
            name,
            parameters,
            clauses,
            soft_behaviors,
            span: Span {
                start: start.start,
                end: self.previous().span.end,
            },
        }))
    }

    fn parameters(&mut self) -> Result<Vec<Parameter>, Vec<Diagnostic>> {
        let mut parameters = Vec::new();
        self.newlines();
        if self.at_symbol(')') {
            self.bump();
            return Ok(parameters);
        }
        loop {
            let name = self.declared_name("expected a parameter name")?;
            self.expect_symbol(':', "MORVA1008", "expected ':' after parameter name")?;
            let type_name = self.type_name("expected a parameter type")?;
            let span = Span::covering(name.span, type_name.span);
            parameters.push(Parameter {
                name,
                type_name,
                span,
            });
            self.newlines();
            if self.at_symbol(')') {
                self.bump();
                break;
            }
            self.expect_symbol(',', "MORVA1009", "expected ',' or ')' after parameter")?;
            self.newlines();
        }
        Ok(parameters)
    }

    fn scenario(&mut self, start: Span, name: Name) -> Result<Declaration, Vec<Diagnostic>> {
        self.expect_block("expected scenario block")?;
        let mut items = Vec::new();
        loop {
            self.newlines();
            if self.at_eof() {
                return Err(self.error("MORVA1003", "unclosed scenario block"));
            }
            if self.at_symbol('}') {
                self.bump();
                break;
            }
            if self.at_identifier("given") {
                self.bump();
                let assignment = self.assignment()?;
                self.line_end("unexpected token after given")?;
                items.push(ScenarioItem::Given(assignment));
            } else if self.at_identifier("run") {
                items.push(ScenarioItem::Run(self.run_item()?));
            } else if self.at_identifier("expect") {
                self.bump();
                let expression = self.expression()?;
                self.line_end("unexpected token after expect")?;
                items.push(ScenarioItem::Expect(expression));
            } else {
                return Err(self.error("MORVA1021", "unknown item in scenario block"));
            }
        }
        Ok(Declaration::Scenario(Scenario {
            name,
            items,
            span: Span {
                start: start.start,
                end: self.previous().span.end,
            },
        }))
    }

    fn run_item(&mut self) -> Result<Run, Vec<Diagnostic>> {
        let start = self.bump().span;
        let action = self.reference_name("expected an action name after run")?;
        self.expect_symbol('(', "MORVA1022", "expected '(' after run action")?;
        let mut arguments = Vec::new();
        self.newlines();
        if !self.at_symbol(')') {
            loop {
                arguments.push(self.declared_name("expected a run argument name")?);
                self.newlines();
                if self.at_symbol(')') {
                    break;
                }
                self.expect_symbol(',', "MORVA1023", "expected ',' or ')' after run argument")?;
                self.newlines();
            }
        }
        self.expect_symbol(')', "MORVA1023", "expected ')' after run arguments")?;
        let end = self.previous().span;
        self.line_end("unexpected token after run")?;
        Ok(Run {
            action,
            arguments,
            span: Span::covering(start, end),
        })
    }

    fn clause(&mut self, kind: ClauseKind) -> Result<Clause, Vec<Diagnostic>> {
        let start = self.bump().span;
        self.newlines();
        let braced = self.at_symbol('{');
        if braced {
            self.bump();
            self.newlines();
        }
        let mut expressions = Vec::new();
        loop {
            if braced && self.at_symbol('}') {
                self.bump();
                break;
            }
            if self.at_eof() || (!braced && (self.at_newline() || self.at_symbol('}'))) {
                break;
            }
            expressions.push(if kind == ClauseKind::Effects {
                ClauseExpression::Assignment(self.assignment()?)
            } else {
                ClauseExpression::Predicate(self.expression()?)
            });
            if braced {
                self.expression_separator()?;
                self.newlines();
            } else {
                self.line_end("unexpected token after clause expression")?;
                break;
            }
        }
        if expressions.is_empty() {
            return Err(self.error("MORVA1010", "clause requires an expression"));
        }
        let end = if braced {
            self.previous().span
        } else {
            expressions.last().expect("non-empty clause").span()
        };
        Ok(Clause {
            kind,
            expressions,
            span: Span::covering(start, end),
        })
    }

    fn assignment(&mut self) -> Result<Assignment, Vec<Diagnostic>> {
        let target = self.path()?;
        let operator = if self.at_symbol('=') {
            self.bump();
            AssignmentOperator::Set
        } else if self.at_operator("+=") {
            self.bump();
            AssignmentOperator::Add
        } else if self.at_operator("-=") {
            self.bump();
            AssignmentOperator::Subtract
        } else {
            return Err(self.error("MORVA1011", "expected assignment operator in effects"));
        };
        let value = self.expression()?;
        Ok(Assignment {
            span: Span::covering(target.span, value.span),
            target,
            operator,
            value,
        })
    }

    fn expression(&mut self) -> Result<Expr, Vec<Diagnostic>> {
        if self.at_symbol('!') {
            let operator_span = self.bump().span;
            let operand = self.expression()?;
            let span = Span::covering(operator_span, operand.span);
            return Ok(Expr {
                kind: ExprKind::Not(Box::new(operand)),
                span,
            });
        }
        if self.at_symbol('(') {
            let opening = self.bump().span;
            let inner = self.expression()?;
            if !self.at_symbol(')') {
                return Err(self.error("MORVA1026", "expected ')' to close the grouped predicate"));
            }
            let closing = self.bump().span;
            return Ok(Expr {
                kind: inner.kind,
                span: Span::covering(opening, closing),
            });
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Expr, Vec<Diagnostic>> {
        let left = self.primary()?;
        let operator = if self.at_operator("==") {
            Some(BinaryOperator::Equal)
        } else if self.at_operator("!=") {
            Some(BinaryOperator::NotEqual)
        } else if self.at_operator(">=") {
            Some(BinaryOperator::GreaterEqual)
        } else if self.at_operator("<=") {
            Some(BinaryOperator::LessEqual)
        } else if self.at_symbol('>') {
            Some(BinaryOperator::Greater)
        } else if self.at_symbol('<') {
            Some(BinaryOperator::Less)
        } else {
            None
        };
        if let Some(operator) = operator {
            self.bump();
            let right = self.primary()?;
            let span = Span::covering(left.span, right.span);
            Ok(Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                span,
            })
        } else {
            Ok(left)
        }
    }

    fn primary(&mut self) -> Result<Expr, Vec<Diagnostic>> {
        let token = self.current().clone();
        match &token.kind {
            TokenKind::Integer(value) => {
                let parsed = value.parse::<i64>().map_err(|_| {
                    vec![Diagnostic::new(
                        "MORVA1012",
                        "integer literal is outside the supported 64-bit range",
                        token.span,
                    )]
                })?;
                self.bump();
                Ok(Expr {
                    kind: ExprKind::Integer(parsed),
                    span: token.span,
                })
            }
            TokenKind::Identifier(value) if value == "true" || value == "false" => {
                self.bump();
                Ok(Expr {
                    kind: ExprKind::Boolean(value == "true"),
                    span: token.span,
                })
            }
            TokenKind::Identifier(_) => {
                let path = self.path()?;
                Ok(Expr {
                    span: path.span,
                    kind: ExprKind::Path(path),
                })
            }
            _ => Err(self.error("MORVA1013", "expected an expression")),
        }
    }

    fn path(&mut self) -> Result<Path, Vec<Diagnostic>> {
        let mut segments = vec![self.reference_name("expected a path")?];
        while self.at_symbol('.') {
            self.bump();
            segments.push(self.reference_name("expected a name after '.'")?);
        }
        let span = Span::covering(
            segments[0].span,
            segments.last().expect("path segment").span,
        );
        Ok(Path { segments, span })
    }

    fn skip_implementation_hint(
        &mut self,
        kind: SoftBehaviorKind,
    ) -> Result<SoftBehavior, Vec<Diagnostic>> {
        let span = self.bump().span;
        self.expect_block("expected block after implementation_hint")?;
        self.skip_balanced_contents()?;
        self.line_end("unexpected token after implementation_hint block")?;
        Ok(SoftBehavior { kind, span })
    }

    fn skip_soft_behavior_line(
        &mut self,
        kind: SoftBehaviorKind,
    ) -> Result<SoftBehavior, Vec<Diagnostic>> {
        let span = self.bump().span;
        while !self.at_eof() && !self.at_newline() && !self.at_symbol('}') {
            if self.at_symbol('{') {
                return Err(self.error("MORVA1014", "unexpected block for soft behavior item"));
            }
            self.bump();
        }
        if self.at_newline() {
            self.bump();
        }
        Ok(SoftBehavior { kind, span })
    }

    fn skip_compatibility_item(&mut self) -> Result<(), Vec<Diagnostic>> {
        while !self.at_eof() && !self.at_newline() && !self.at_symbol('}') {
            if self.at_symbol('{') {
                self.bump();
                self.skip_balanced_contents()?;
                break;
            }
            self.bump();
        }
        if self.at_newline() {
            self.bump();
        }
        Ok(())
    }

    fn skip_balanced_contents(&mut self) -> Result<(), Vec<Diagnostic>> {
        let opening = self.previous().span;
        let mut depth = 1usize;
        while !self.at_eof() {
            if self.at_symbol('{') {
                depth += 1;
            } else if self.at_symbol('}') {
                depth -= 1;
                if depth == 0 {
                    self.bump();
                    return Ok(());
                }
            }
            self.bump();
        }
        Err(vec![Diagnostic::new(
            "MORVA1003",
            "unclosed compatibility block",
            opening,
        )])
    }

    fn skip_signature(&mut self) -> Result<(), Vec<Diagnostic>> {
        let mut depth = 0usize;
        while !self.at_eof() {
            if self.at_symbol('{') && depth == 0 {
                return Ok(());
            }
            if depth == 0 && self.current_declaration_kind().is_some() {
                return Err(self.error("MORVA1016", "expected declaration block"));
            }
            if self.at_newline() {
                if depth == 0 {
                    self.newlines();
                    return if self.at_symbol('{') {
                        Ok(())
                    } else {
                        Err(self.error("MORVA1016", "expected declaration block"))
                    };
                }
                self.bump();
                continue;
            }
            if self.at_symbol('(') {
                depth += 1;
            } else if self.at_symbol(')') {
                if depth == 0 {
                    return Err(self.error("MORVA1015", "unexpected ')'"));
                }
                depth -= 1;
            } else if self.at_symbol('}') && depth == 0 {
                return Err(self.error("MORVA1016", "expected declaration block"));
            }
            self.bump();
        }
        Err(self.error("MORVA1016", "expected declaration block"))
    }

    fn expect_block(&mut self, message: &str) -> Result<(), Vec<Diagnostic>> {
        self.newlines();
        if self.at_symbol('{') {
            self.bump();
            Ok(())
        } else {
            Err(self.error("MORVA1016", message))
        }
    }

    fn expression_separator(&mut self) -> Result<(), Vec<Diagnostic>> {
        if self.at_symbol(';') {
            self.bump();
        }
        if self.at_newline() || self.at_symbol('}') {
            Ok(())
        } else {
            Err(self.error("MORVA1017", "expected a line break between expressions"))
        }
    }

    fn line_end(&mut self, message: &str) -> Result<(), Vec<Diagnostic>> {
        if self.at_symbol(';') {
            self.bump();
        }
        if self.at_newline() {
            self.bump();
            Ok(())
        } else if self.at_symbol('}') || self.at_eof() {
            Ok(())
        } else {
            Err(self.error("MORVA1018", message))
        }
    }

    fn declared_name(&mut self, message: &str) -> Result<Name, Vec<Diagnostic>> {
        let name = self.reference_name(message)?;
        if RESERVED_NAMES.contains(&name.text.as_str()) {
            Err(vec![Diagnostic::new(
                "MORVA1019",
                format!("'{}' is reserved and cannot be used as a name", name.text),
                name.span,
            )])
        } else {
            Ok(name)
        }
    }

    fn type_name(&mut self, message: &str) -> Result<Name, Vec<Diagnostic>> {
        self.reference_name(message)
    }

    fn reference_name(&mut self, message: &str) -> Result<Name, Vec<Diagnostic>> {
        let token = self.current().clone();
        if let TokenKind::Identifier(text) = token.kind {
            self.bump();
            Ok(Name {
                text,
                span: token.span,
            })
        } else {
            Err(self.error("MORVA1020", message))
        }
    }

    fn expect_symbol(
        &mut self,
        symbol: char,
        code: &'static str,
        message: &str,
    ) -> Result<(), Vec<Diagnostic>> {
        if self.at_symbol(symbol) {
            self.bump();
            Ok(())
        } else {
            Err(self.error(code, message))
        }
    }

    fn current_declaration_kind(&self) -> Option<&str> {
        let value = identifier_text(self.current())?;
        (SEMANTIC_DECLARATION_KINDS.contains(&value)
            || COMPATIBILITY_CONTAINER_KINDS.contains(&value))
        .then_some(value)
    }

    fn current_soft_behavior_kind(&self) -> Option<SoftBehaviorKind> {
        match identifier_text(self.current())? {
            "atomic" => Some(SoftBehaviorKind::Atomic),
            "idempotent" => Some(SoftBehaviorKind::Idempotent),
            "timeout" => Some(SoftBehaviorKind::Timeout),
            "retry" => Some(SoftBehaviorKind::Retry),
            _ => None,
        }
    }

    fn at_identifier(&self, expected: &str) -> bool {
        matches!(&self.current().kind, TokenKind::Identifier(value) if value == expected)
    }

    fn at_operator(&self, expected: &str) -> bool {
        matches!(&self.current().kind, TokenKind::Operator(value) if *value == expected)
    }

    fn at_symbol(&self, expected: char) -> bool {
        self.current().kind == TokenKind::Symbol(expected)
    }

    fn at_newline(&self) -> bool {
        self.current().kind == TokenKind::Newline
    }

    fn at_eof(&self) -> bool {
        self.current().kind == TokenKind::Eof
    }

    fn newlines(&mut self) {
        while self.at_newline() {
            self.bump();
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.cursor - 1]
    }

    fn bump(&mut self) -> &Token {
        let index = self.cursor;
        if !self.at_eof() {
            self.cursor += 1;
        }
        &self.tokens[index]
    }

    fn error(&self, code: &'static str, message: &str) -> Vec<Diagnostic> {
        vec![Diagnostic::new(code, message, self.current().span)]
    }
}

fn identifier_text(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Identifier(value) => Some(value),
        _ => None,
    }
}
