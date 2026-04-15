//! # Evaluator
//!
//! Executes tiny-talk programs by walking the AST.
//!
//! ## Responsibilities
//!
//! - **AST traversal**: Walk the abstract syntax tree, visiting each node and
//!   performing the corresponding computation.
//!
//! - **Message dispatch**: When evaluating a message send, look up the method
//!   in the receiver's class hierarchy and invoke it.
//!
//! - **Environment management**: Maintain the mapping from variable names to
//!   values, handling lexical scoping and block closures.
//!
//! - **Control flow**: Implement Smalltalk's control flow through message sends
//!   (blocks receiving `ifTrue:ifFalse:`, `whileTrue:`, etc.).
//!
//! - **Stack management**: Track the call stack for method invocations, enabling
//!   proper returns and useful error messages.
//!
//! - **Error handling**: Detect and report runtime errors (message not understood,
//!   type errors, etc.) with accurate source locations.
//!
//! ## Dependencies and Relationships
//!
//! This module depends on:
//! - `ast`: for AST node definitions to traverse
//! - `vm`: for object manipulation, method dispatch, and primitive operations
//! - `source`: for location information in error messages
//! - `diagnostics`: for runtime error reporting
//!
//! This module is used by:
//! - The CLI: to run tiny-talk programs
//!
//! ## Architectural Approach
//!
//! ### Tree-Walking Interpretation
//!
//! The evaluator directly interprets the AST—no compilation to bytecode or other
//! intermediate form. This is:
//!
//! - **Simple**: Easy to implement and understand
//! - **Direct**: Clear correspondence between syntax and behavior
//! - **Slow**: Not suitable for production, but fine for learning
//!
//! Each AST node type has corresponding evaluation logic.
//!
//! ### Evaluation Result
//!
//! Evaluating an expression produces a **value** (an object handle). The evaluator
//! functions return this handle, propagating it up through the tree.
//!
//! Special cases:
//! - **Errors**: Evaluation can fail; we use Rust's `Result` type
//! - **Returns**: The `^` operator causes a non-local return; this needs special
//!   handling (exception-like mechanism or explicit checking)
//!
//! ### Environment and Scope
//!
//! Variables are looked up in an **environment**—a mapping from names to handles.
//! Environments are chained for nested scopes:
//!
//! - Method activation creates a new environment
//! - Block closures capture their lexical environment
//! - Variable lookup walks the environment chain
//!
//! This implements Smalltalk's lexical scoping.
//!
//! ### Message Send Evaluation
//!
//! Evaluating `receiver message: arg1 with: arg2`:
//!
//! 1. Evaluate `receiver` to get a handle
//! 2. Evaluate `arg1` and `arg2` to get argument handles
//! 3. Look up `message:with:` in the receiver's class hierarchy
//! 4. If found, create a new activation with bound parameters
//! 5. Evaluate the method body in that activation
//! 6. Return the method's result
//!
//! ### Block Closures
//!
//! Blocks are first-class functions. When a block literal is evaluated:
//!
//! 1. Create a closure object capturing the current environment
//! 2. Store the block's AST (parameters and body)
//! 3. Return the closure as a value
//!
//! When the closure receives `value:` (or similar):
//!
//! 1. Extend the captured environment with parameter bindings
//! 2. Evaluate the block body
//! 3. Return the result
//!
//! ### Non-Local Returns
//!
//! Smalltalk's `^` returns from the **enclosing method**, not just the block:
//!
//! ```smalltalk
//! myMethod
//!     1 to: 10 do: [:i | i > 5 ifTrue: [^i]].
//!     ^0
//! ```
//!
//! This returns from `myMethod` when `i > 5`, not just from the block. We implement
//! this with an exception-like mechanism: `^` throws a "return" signal caught by
//! the method boundary.
//!
//! ### Primitives
//!
//! Some operations can't be expressed in tiny-talk and need Rust implementations:
//!
//! - Arithmetic: `+`, `-`, `*`, `/` on numbers
//! - Comparison: `<`, `>`, `=`
//! - I/O: `print`, reading input
//! - Object creation: `new`, `basicNew`
//!
//! The evaluator checks if a method is primitive and calls the Rust implementation
//! instead of evaluating AST.
//!
//! ### Error Reporting
//!
//! When evaluation fails, we produce a diagnostic with:
//!
//! - The error message (e.g., "message #foo not understood by SmallInteger")
//! - The source location where the error occurred
//! - Optionally, a stack trace showing the call chain

// TODO: Implement evaluator
