//! # Abstract Syntax Tree
//!
//! Defines the hierarchical representation of program structure.
//!
//! ## Responsibilities
//!
//! - **Syntax representation**: Define node types for all syntactic constructs in
//!   tiny-talk (expressions, statements, method definitions, class definitions).
//!
//! - **Tree structure**: Represent the hierarchical nesting of constructs (blocks
//!   containing statements, message sends containing receivers and arguments).
//!
//! - **Source tracking**: Every AST node carries its source span, enabling the
//!   evaluator and diagnostic system to report precise error locations.
//!
//! - **Traversal support**: Provide mechanisms for walking the tree (visitor pattern
//!   or direct recursive traversal) for evaluation and analysis.
//!
//! ## Dependencies and Relationships
//!
//! This module depends on:
//! - `source`: for span types to track node locations
//! - `lexeme`: may embed tokens for precise location info (optional approach)
//!
//! This module is used by:
//! - `parser`: constructs AST nodes from the token stream
//! - `eval`: walks the AST to execute the program
//!
//! ## Architectural Approach
//!
//! ### Node Design
//!
//! AST nodes are typically implemented as an enum with variants for each construct,
//! or as a trait with concrete types. For tiny-talk, we use an **enum-based**
//! approach for its simplicity and exhaustive pattern matching.
//!
//! Key node categories:
//!
//! #### Expressions
//! - **Literals**: Numbers, strings, symbols, characters, booleans, nil
//! - **Variable references**: Reading a variable's value
//! - **Assignments**: `variable := expression`
//! - **Message sends**: Unary, binary, and keyword messages
//! - **Cascades**: `receiver msg1; msg2; msg3`
//! - **Blocks**: `[:args | statements]`
//! - **Array literals**: `#(element element ...)`
//!
//! #### Statements
//! - **Expression statements**: An expression evaluated for side effects
//! - **Return statements**: `^expression` to return from a method
//!
//! #### Definitions (for later phases)
//! - **Method definitions**: Selector, arguments, temporaries, body
//! - **Class definitions**: Name, superclass, instance variables, methods
//!
//! ### Smalltalk Message Syntax
//!
//! Smalltalk has three kinds of messages, and their parsing precedence matters:
//!
//! 1. **Unary messages** (highest precedence): `receiver message`
//! 2. **Binary messages** (middle precedence): `receiver + argument`
//! 3. **Keyword messages** (lowest precedence): `receiver at: key put: value`
//!
//! Example: `3 factorial + 4 squared` parses as `(3 factorial) + (4 squared)`
//!
//! This precedence is encoded in the AST structure—the parser handles it, and the
//! AST simply represents the result.
//!
//! ### Blocks as Values
//!
//! Blocks are first-class values in Smalltalk. They capture:
//! - Parameter names (e.g., `[:a :b | ...]` has parameters `a` and `b`)
//! - Local temporaries (`[:a | |temp| ...]`)
//! - The body (a sequence of statements)
//! - Their lexical environment (handled at runtime, not in the AST)
//!
//! ### Source Spans
//!
//! Each node stores its span (start and end positions in source). For compound
//! nodes, the span covers from the first token to the last token of the construct.
//! This enables error messages like:
//!
//! ```text
//! error: message not understood: #frobnicate
//!   --> example.tt:10:5
//!    |
//! 10 |     myObject frobnicate: 42
//!    |     ^^^^^^^^^^^^^^^^^^^^^^^
//! ```
//!
//! ### No Parent Pointers
//!
//! AST nodes do not contain pointers back to their parents. Tree traversal is
//! top-down, with context passed explicitly. This simplifies memory management
//! and makes the AST easy to construct and transform.

use crate::lexeme::TokenKind;
use crate::source::Span;

// ============================================================================
// Program and Statements
// ============================================================================

/// A parsed tiny-talk compilation unit.
///
/// The initial language shape is script-oriented: a program is a sequence of
/// statements with one overall source span. Class and method declarations are
/// modeled separately below so later parser work can reuse the same data types
/// without changing the expression tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Top-level statements in source order.
    pub statements: Vec<Statement>,
    /// The source span covered by the program.
    pub span: Span,
}

impl Program {
    /// Creates a new program node.
    pub fn new(statements: Vec<Statement>, span: Span) -> Self {
        Self { statements, span }
    }

    /// Returns `true` when this program has no statements.
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }
}

/// A statement in a method, block, or top-level script body.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// An expression evaluated for its value or side effects.
    Expression(Expression),
    /// A Smalltalk return statement: `^ expression`.
    Return(ReturnStatement),
}

impl Statement {
    /// Returns the source span for this statement.
    pub fn span(&self) -> Span {
        match self {
            Statement::Expression(expression) => expression.span(),
            Statement::Return(statement) => statement.span,
        }
    }
}

/// A return statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement {
    /// The value being returned.
    pub value: Expression,
    /// The span from the caret through the returned expression.
    pub span: Span,
}

impl ReturnStatement {
    /// Creates a return statement.
    pub fn new(value: Expression, span: Span) -> Self {
        Self { value, span }
    }
}

// ============================================================================
// Expressions
// ============================================================================

/// An expression node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// A literal value, such as a number, string, symbol, boolean, or `nil`.
    Literal(LiteralExpression),
    /// A variable or pseudo-variable reference.
    Variable(VariableExpression),
    /// An assignment: `name := value`.
    Assignment(AssignmentExpression),
    /// A message send with an explicit receiver.
    MessageSend(MessageSendExpression),
    /// A cascade send: `receiver first; second: arg`.
    Cascade(CascadeExpression),
    /// A block literal: `[:arg | statements]`.
    Block(BlockExpression),
    /// A literal array: `#(element element ...)`.
    ArrayLiteral(ArrayLiteralExpression),
}

impl Expression {
    /// Creates a literal expression.
    pub fn literal(value: LiteralValue, span: Span) -> Self {
        Expression::Literal(LiteralExpression::new(value, span))
    }

    /// Creates a variable reference expression.
    pub fn variable(name: impl Into<String>, span: Span) -> Self {
        Expression::Variable(VariableExpression::new(name, span))
    }

    /// Creates an assignment expression.
    pub fn assignment(name: impl Into<String>, value: Expression, span: Span) -> Self {
        Expression::Assignment(AssignmentExpression::new(name, value, span))
    }

    /// Creates a message send expression.
    pub fn message_send(receiver: Expression, message: Message, span: Span) -> Self {
        Expression::MessageSend(MessageSendExpression::new(receiver, message, span))
    }

    /// Creates a cascade expression.
    pub fn cascade(receiver: Expression, messages: Vec<Message>, span: Span) -> Self {
        Expression::Cascade(CascadeExpression::new(receiver, messages, span))
    }

    /// Creates a block expression.
    pub fn block(
        parameters: Vec<String>,
        temporaries: Vec<String>,
        body: Vec<Statement>,
        span: Span,
    ) -> Self {
        Expression::Block(BlockExpression::new(parameters, temporaries, body, span))
    }

    /// Creates a literal array expression.
    pub fn array_literal(elements: Vec<LiteralValue>, span: Span) -> Self {
        Expression::ArrayLiteral(ArrayLiteralExpression::new(elements, span))
    }

    /// Returns the source span for this expression.
    pub fn span(&self) -> Span {
        match self {
            Expression::Literal(expression) => expression.span,
            Expression::Variable(expression) => expression.span,
            Expression::Assignment(expression) => expression.span,
            Expression::MessageSend(expression) => expression.span,
            Expression::Cascade(expression) => expression.span,
            Expression::Block(expression) => expression.span,
            Expression::ArrayLiteral(expression) => expression.span,
        }
    }
}

/// A literal expression.
#[derive(Debug, Clone, PartialEq)]
pub struct LiteralExpression {
    /// The literal's semantic value.
    pub value: LiteralValue,
    /// The span covering the literal token.
    pub span: Span,
}

impl LiteralExpression {
    /// Creates a literal expression.
    pub fn new(value: LiteralValue, span: Span) -> Self {
        Self { value, span }
    }
}

/// Values that can appear directly as literals in source.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    /// An integer literal.
    Integer(i64),
    /// A floating-point literal.
    Float(f64),
    /// A string literal.
    String(String),
    /// A symbol literal.
    Symbol(String),
    /// A character literal.
    Character(char),
    /// A boolean pseudo-literal (`true` or `false`).
    Boolean(bool),
    /// The `nil` pseudo-literal.
    Nil,
}

impl LiteralValue {
    /// Converts a lexical token kind into an AST literal value when possible.
    ///
    /// The lexer represents `true`, `false`, and `nil` as identifiers because
    /// Smalltalk treats them as pseudo-variables rather than reserved words. The
    /// AST stores them as literal values so later evaluation can recognize them
    /// without stringly-typed special cases.
    pub fn from_token_kind(kind: &TokenKind) -> Option<Self> {
        match kind {
            TokenKind::Integer(value) => Some(LiteralValue::Integer(*value)),
            TokenKind::Float(value) => Some(LiteralValue::Float(*value)),
            TokenKind::String(value) => Some(LiteralValue::String(value.clone())),
            TokenKind::Symbol(value) => Some(LiteralValue::Symbol(value.clone())),
            TokenKind::Character(value) => Some(LiteralValue::Character(*value)),
            TokenKind::Identifier(value) if value == "true" => Some(LiteralValue::Boolean(true)),
            TokenKind::Identifier(value) if value == "false" => Some(LiteralValue::Boolean(false)),
            TokenKind::Identifier(value) if value == "nil" => Some(LiteralValue::Nil),
            _ => None,
        }
    }
}

/// A variable reference.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableExpression {
    /// The referenced variable name.
    pub name: String,
    /// The span covering the identifier.
    pub span: Span,
}

impl VariableExpression {
    /// Creates a variable reference.
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

/// An assignment expression.
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpression {
    /// The variable being assigned.
    pub target: String,
    /// The expression assigned to the target.
    pub value: Box<Expression>,
    /// The span covering the full assignment.
    pub span: Span,
}

impl AssignmentExpression {
    /// Creates an assignment expression.
    pub fn new(target: impl Into<String>, value: Expression, span: Span) -> Self {
        Self {
            target: target.into(),
            value: Box::new(value),
            span,
        }
    }
}

/// A message send expression.
#[derive(Debug, Clone, PartialEq)]
pub struct MessageSendExpression {
    /// The receiver expression.
    pub receiver: Box<Expression>,
    /// The selector and arguments being sent.
    pub message: Message,
    /// The span covering the full send.
    pub span: Span,
}

impl MessageSendExpression {
    /// Creates a message send expression.
    pub fn new(receiver: Expression, message: Message, span: Span) -> Self {
        Self {
            receiver: Box::new(receiver),
            message,
            span,
        }
    }
}

/// A cascade expression.
///
/// Cascades send multiple messages to the same receiver. The receiver is stored
/// once, and each cascade leg is represented by a [`Message`] without duplicating
/// the receiver expression.
#[derive(Debug, Clone, PartialEq)]
pub struct CascadeExpression {
    /// The shared receiver for every message in the cascade.
    pub receiver: Box<Expression>,
    /// Messages sent to the shared receiver, in source order.
    pub messages: Vec<Message>,
    /// The span covering the full cascade expression.
    pub span: Span,
}

impl CascadeExpression {
    /// Creates a cascade expression.
    pub fn new(receiver: Expression, messages: Vec<Message>, span: Span) -> Self {
        Self {
            receiver: Box::new(receiver),
            messages,
            span,
        }
    }
}

/// A block literal expression.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockExpression {
    /// Block parameter names, without their leading colons.
    pub parameters: Vec<String>,
    /// Local temporary variable names declared by the block.
    pub temporaries: Vec<String>,
    /// Statements in the block body.
    pub body: Vec<Statement>,
    /// The span covering the block brackets and contents.
    pub span: Span,
}

impl BlockExpression {
    /// Creates a block expression.
    pub fn new(
        parameters: Vec<String>,
        temporaries: Vec<String>,
        body: Vec<Statement>,
        span: Span,
    ) -> Self {
        Self {
            parameters,
            temporaries,
            body,
            span,
        }
    }
}

/// A literal array expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteralExpression {
    /// Literal elements in source order.
    pub elements: Vec<LiteralValue>,
    /// The span covering the array literal.
    pub span: Span,
}

impl ArrayLiteralExpression {
    /// Creates a literal array expression.
    pub fn new(elements: Vec<LiteralValue>, span: Span) -> Self {
        Self { elements, span }
    }
}

// ============================================================================
// Messages and Selectors
// ============================================================================

/// A message selector with its syntactic category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// A unary selector, such as `size`.
    Unary(String),
    /// A binary selector, such as `+`.
    Binary(String),
    /// A keyword selector split into source-order parts, such as `at:` and `put:`.
    Keyword(Vec<String>),
}

impl Selector {
    /// Returns the selector as the runtime dispatch name.
    pub fn name(&self) -> String {
        match self {
            Selector::Unary(name) | Selector::Binary(name) => name.clone(),
            Selector::Keyword(parts) => parts.concat(),
        }
    }

    /// Returns the number of arguments expected by this selector.
    pub fn arity(&self) -> usize {
        match self {
            Selector::Unary(_) => 0,
            Selector::Binary(_) => 1,
            Selector::Keyword(parts) => parts.len(),
        }
    }

    /// Returns `true` for unary selectors.
    pub fn is_unary(&self) -> bool {
        matches!(self, Selector::Unary(_))
    }

    /// Returns `true` for binary selectors.
    pub fn is_binary(&self) -> bool {
        matches!(self, Selector::Binary(_))
    }

    /// Returns `true` for keyword selectors.
    pub fn is_keyword(&self) -> bool {
        matches!(self, Selector::Keyword(_))
    }
}

/// A message send without its receiver.
///
/// This shape is used both by ordinary message sends and by cascade legs.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// The selector being sent.
    pub selector: Selector,
    /// Argument expressions, in selector order.
    pub arguments: Vec<Expression>,
    /// The span covering the message selector and arguments, excluding the receiver.
    pub span: Span,
}

impl Message {
    /// Creates a message from a selector and arguments.
    pub fn new(selector: Selector, arguments: Vec<Expression>, span: Span) -> Self {
        Self {
            selector,
            arguments,
            span,
        }
    }

    /// Creates a unary message.
    pub fn unary(selector: impl Into<String>, span: Span) -> Self {
        Self::new(Selector::Unary(selector.into()), Vec::new(), span)
    }

    /// Creates a binary message.
    pub fn binary(selector: impl Into<String>, argument: Expression, span: Span) -> Self {
        Self::new(Selector::Binary(selector.into()), vec![argument], span)
    }

    /// Creates a keyword message from selector parts and arguments.
    pub fn keyword(parts: Vec<String>, arguments: Vec<Expression>, span: Span) -> Self {
        Self::new(Selector::Keyword(parts), arguments, span)
    }

    /// Returns the runtime dispatch selector name.
    pub fn selector_name(&self) -> String {
        self.selector.name()
    }

    /// Returns the number of arguments supplied by this message.
    pub fn arity(&self) -> usize {
        self.arguments.len()
    }
}

// ============================================================================
// Definitions
// ============================================================================

/// A method definition.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDefinition {
    /// The method selector.
    pub selector: Selector,
    /// Formal parameter names in selector order.
    pub parameters: Vec<String>,
    /// Method-local temporary variable names.
    pub temporaries: Vec<String>,
    /// Method body statements.
    pub body: Vec<Statement>,
    /// The span covering the method definition.
    pub span: Span,
}

impl MethodDefinition {
    /// Creates a method definition.
    pub fn new(
        selector: Selector,
        parameters: Vec<String>,
        temporaries: Vec<String>,
        body: Vec<Statement>,
        span: Span,
    ) -> Self {
        Self {
            selector,
            parameters,
            temporaries,
            body,
            span,
        }
    }
}

/// A class definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDefinition {
    /// The class name.
    pub name: String,
    /// The optional superclass name.
    pub superclass: Option<String>,
    /// Instance variable names declared by this class.
    pub instance_variables: Vec<String>,
    /// Methods defined directly on this class.
    pub methods: Vec<MethodDefinition>,
    /// The span covering the class definition.
    pub span: Span,
}

impl ClassDefinition {
    /// Creates a class definition.
    pub fn new(
        name: impl Into<String>,
        superclass: Option<String>,
        instance_variables: Vec<String>,
        methods: Vec<MethodDefinition>,
        span: Span,
    ) -> Self {
        Self {
            name: name.into(),
            superclass,
            instance_variables,
            methods,
            span,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::SourceId;

    fn span(start: u32, len: u32) -> Span {
        Span::new(SourceId::SYNTHETIC, start, len)
    }

    #[test]
    fn literal_values_convert_from_lexeme_token_kinds() {
        assert_eq!(
            LiteralValue::from_token_kind(&TokenKind::Integer(42)),
            Some(LiteralValue::Integer(42))
        );
        assert_eq!(
            LiteralValue::from_token_kind(&TokenKind::Float(3.5)),
            Some(LiteralValue::Float(3.5))
        );
        assert_eq!(
            LiteralValue::from_token_kind(&TokenKind::String("hello".into())),
            Some(LiteralValue::String("hello".into()))
        );
        assert_eq!(
            LiteralValue::from_token_kind(&TokenKind::Symbol("answer".into())),
            Some(LiteralValue::Symbol("answer".into()))
        );
        assert_eq!(
            LiteralValue::from_token_kind(&TokenKind::Character('x')),
            Some(LiteralValue::Character('x'))
        );
        assert_eq!(
            LiteralValue::from_token_kind(&TokenKind::Identifier("true".into())),
            Some(LiteralValue::Boolean(true))
        );
        assert_eq!(
            LiteralValue::from_token_kind(&TokenKind::Identifier("false".into())),
            Some(LiteralValue::Boolean(false))
        );
        assert_eq!(
            LiteralValue::from_token_kind(&TokenKind::Identifier("nil".into())),
            Some(LiteralValue::Nil)
        );
        assert_eq!(
            LiteralValue::from_token_kind(&TokenKind::Identifier("ordinary".into())),
            None
        );
    }

    #[test]
    fn expression_and_statement_spans_report_their_node_spans() {
        let expression_span = span(2, 3);
        let expression = Expression::variable("receiver", expression_span);
        assert_eq!(expression.span(), expression_span);

        let return_span = span(1, 4);
        let statement = Statement::Return(ReturnStatement::new(expression, return_span));
        assert_eq!(statement.span(), return_span);
    }

    #[test]
    fn selectors_preserve_category_name_and_arity() {
        let unary = Selector::Unary("size".into());
        assert!(unary.is_unary());
        assert_eq!(unary.name(), "size");
        assert_eq!(unary.arity(), 0);

        let binary = Selector::Binary("+".into());
        assert!(binary.is_binary());
        assert_eq!(binary.name(), "+");
        assert_eq!(binary.arity(), 1);

        let keyword = Selector::Keyword(vec!["at:".into(), "put:".into()]);
        assert!(keyword.is_keyword());
        assert_eq!(keyword.name(), "at:put:");
        assert_eq!(keyword.arity(), 2);
    }

    #[test]
    fn message_send_stores_receiver_selector_and_arguments() {
        let receiver = Expression::variable("array", span(0, 5));
        let index = Expression::literal(LiteralValue::Integer(1), span(9, 1));
        let value = Expression::literal(LiteralValue::String("value".into()), span(16, 7));
        let message = Message::keyword(
            vec!["at:".into(), "put:".into()],
            vec![index, value],
            span(6, 17),
        );

        let send = Expression::message_send(receiver, message, span(0, 23));
        match send {
            Expression::MessageSend(send) => {
                assert_eq!(send.message.selector_name(), "at:put:");
                assert_eq!(send.message.arity(), 2);
                assert_eq!(send.span, span(0, 23));
            }
            other => panic!("expected message send, got {other:?}"),
        }
    }

    #[test]
    fn cascade_stores_shared_receiver_and_ordered_messages() {
        let receiver = Expression::variable("builder", span(0, 7));
        let first = Message::unary("reset", span(8, 5));
        let second = Message::binary(
            "+",
            Expression::literal(LiteralValue::Integer(1), span(16, 1)),
            span(14, 3),
        );

        let cascade = Expression::cascade(receiver, vec![first, second], span(0, 17));
        match cascade {
            Expression::Cascade(cascade) => {
                assert_eq!(cascade.messages.len(), 2);
                assert_eq!(cascade.messages[0].selector_name(), "reset");
                assert_eq!(cascade.messages[1].selector_name(), "+");
            }
            other => panic!("expected cascade, got {other:?}"),
        }
    }

    #[test]
    fn block_records_parameters_temporaries_and_body() {
        let body_expression = Expression::assignment(
            "sum",
            Expression::variable("value", span(15, 5)),
            span(8, 12),
        );
        let block = Expression::block(
            vec!["value".into()],
            vec!["sum".into()],
            vec![Statement::Expression(body_expression)],
            span(0, 21),
        );

        match block {
            Expression::Block(block) => {
                assert_eq!(block.parameters, vec!["value"]);
                assert_eq!(block.temporaries, vec!["sum"]);
                assert_eq!(block.body.len(), 1);
                assert_eq!(block.span, span(0, 21));
            }
            other => panic!("expected block, got {other:?}"),
        }
    }

    #[test]
    fn definitions_share_statement_and_selector_shapes() {
        let return_statement = Statement::Return(ReturnStatement::new(
            Expression::literal(LiteralValue::Nil, span(10, 3)),
            span(8, 5),
        ));
        let method = MethodDefinition::new(
            Selector::Unary("initialize".into()),
            Vec::new(),
            vec!["tmp".into()],
            vec![return_statement],
            span(0, 13),
        );
        let class = ClassDefinition::new(
            "Point",
            Some("Object".into()),
            vec!["x".into(), "y".into()],
            vec![method],
            span(0, 42),
        );

        assert_eq!(class.name, "Point");
        assert_eq!(class.superclass, Some("Object".into()));
        assert_eq!(class.methods[0].selector.name(), "initialize");
        assert_eq!(class.methods[0].body.len(), 1);
    }
}

