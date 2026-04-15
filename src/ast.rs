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

// TODO: Define AST node types
