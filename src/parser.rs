//! # Parser
//!
//! Transforms the token stream into an abstract syntax tree.
//!
//! ## Responsibilities
//!
//! - **Grammar implementation**: Implement the grammar rules of tiny-talk,
//!   recognizing valid syntactic structures from the flat token sequence.
//!
//! - **AST construction**: Build AST nodes as grammar rules are matched,
//!   assembling the hierarchical program representation.
//!
//! - **Precedence handling**: Correctly handle Smalltalk's message precedence
//!   (unary > binary > keyword) and left-to-right associativity.
//!
//! - **Error recovery**: When syntax errors occur, recover gracefully to continue
//!   parsing and report multiple errors rather than stopping at the first.
//!
//! - **Error reporting**: Emit clear diagnostic messages for syntax errors,
//!   with suggestions when possible.
//!
//! ## Dependencies and Relationships
//!
//! This module depends on:
//! - `lexeme`: for token type definitions and token inspection
//! - `ast`: for AST node construction
//! - `source`: for span manipulation
//! - `diagnostics`: for error reporting
//!
//! This module is used by:
//! - `eval`: receives the AST for execution
//! - The CLI: parses source files before evaluation
//!
//! ## Architectural Approach
//!
//! ### Parsing Strategy
//!
//! The parser uses **recursive descent**, a straightforward top-down approach where
//! each grammar rule corresponds to a function. This is well-suited to Smalltalk's
//! relatively simple grammar and makes the precedence handling explicit.
//!
//! The main entry point parses a sequence of statements (for script-style input)
//! or a method/class definition (for structured input).
//!
//! ### Smalltalk Grammar Sketch
//!
//! ```text
//! script       := statement*
//! statement    := expression '.' | return_stmt
//! return_stmt  := '^' expression '.'?
//!
//! expression   := assignment | cascade
//! assignment   := identifier ':=' expression
//! cascade      := keyword_send (';' message)*
//!
//! keyword_send := binary_send (keyword binary_send)*
//! binary_send  := unary_send (binary_op unary_send)*
//! unary_send   := primary unary_msg*
//!
//! primary      := literal | identifier | block | '(' expression ')'
//! block        := '[' block_args? block_body ']'
//! block_args   := (':' identifier)+ '|'
//! block_body   := temporaries? statement*
//! temporaries  := '|' identifier* '|'
//! ```
//!
//! ### Message Precedence
//!
//! The grammar structure enforces Smalltalk's precedence rules:
//!
//! - `unary_send` is parsed tightest (multiple unary messages chain left-to-right)
//! - `binary_send` parses binary operators between unary sends
//! - `keyword_send` collects keyword parts around binary sends
//!
//! Example: `a b + c d: e f` parses as `((a b) + c) d: (e f)`
//!
//! ### Lookahead and Token Consumption
//!
//! The parser maintains a cursor into the token stream. Key operations:
//!
//! - **peek**: Look at the current token without consuming
//! - **advance**: Move to the next token
//! - **expect**: Consume a token of a specific kind, or emit an error
//! - **check**: Test if the current token matches a kind (for optional elements)
//!
//! Smalltalk's grammar is mostly LL(1)—one token of lookahead suffices for most
//! decisions. The main exception is distinguishing identifiers from keywords
//! (which requires checking for a following colon).
//!
//! ### Error Recovery
//!
//! When a syntax error occurs, the parser:
//!
//! 1. Emits a diagnostic describing the expected vs found tokens
//! 2. Attempts **synchronization**: skipping tokens until a safe resumption point
//! 3. Continues parsing from the synchronized position
//!
//! Good synchronization points include:
//! - Statement boundaries (`.` or end of block)
//! - Block/parenthesis closers (`]`, `)`)
//! - The end of file
//!
//! This allows reporting multiple errors in a single parse attempt.
//!
//! ### Span Computation
//!
//! As the parser builds AST nodes, it computes spans by combining the spans of
//! constituent tokens/nodes. A utility function merges two spans into one covering
//! both, which is used repeatedly during tree construction.

// TODO: Implement parser
