//! # tiny-talk
//!
//! A minimal interpreter for a Smalltalk-like language.
//!
//! ## Overview
//!
//! tiny-talk explores the core concepts that make Smalltalk elegant:
//!
//! - **Pervasive message-passing**: Everything is an object, and computation
//!   happens by sending messages to objects.
//! - **"Turtles all the way down" OOP**: Objects, classes, and even the runtime
//!   itself follow the same consistent model.
//!
//! This crate provides both a library for embedding the interpreter and a
//! command-line interface for running tiny-talk programs.
//!
//! ## Architecture
//!
//! The interpreter is organized as a pipeline that transforms source code into
//! executable behavior:
//!
//! ```text
//! Source Code → Lexer → Tokens → Parser → AST → Evaluator → Results
//!                                                   ↓
//!                                                  VM
//! ```
//!
//! ### Module Organization
//!
//! The crate is divided into focused modules, each handling a specific stage of
//! interpretation or a cross-cutting concern:
//!
//! #### Foundation Modules
//!
//! - **[`source`]**: Manages source files and tracks locations within them.
//!   Provides the coordinate system (file, line, column, spans) used by all
//!   other modules to report positions in user code. Source files are owned
//!   centrally by a source manager, with lightweight IDs used throughout.
//!
//! - **[`diagnostics`]**: Unified error and warning reporting. All modules emit
//!   diagnostics through this system, which handles formatting, severity levels,
//!   and presentation. Depends on `source` for location information.
//!
//! #### Lexical Analysis
//!
//! - **[`lexeme`]**: Defines the vocabulary of the language—tokens (semantically
//!   meaningful units like identifiers and operators) and trivia (whitespace,
//!   comments). Each lexeme carries its source location for error reporting.
//!
//! - **[`lexer`]**: Transforms source text into a sequence of lexemes. Handles
//!   character-level concerns like recognizing keywords, operators, and literals.
//!   Produces a flat vector of tokens (trivia filtered out for parsing).
//!
//! #### Syntax Analysis
//!
//! - **[`ast`]**: Defines the abstract syntax tree—the hierarchical representation
//!   of program structure. Nodes represent constructs like message sends, blocks,
//!   and method definitions. Each node tracks its source span.
//!
//! - **[`parser`]**: Transforms the token stream into an AST. Implements the
//!   grammar rules of tiny-talk and produces structured syntax trees from flat
//!   token sequences.
//!
//! #### Runtime
//!
//! - **[`vm`]**: The virtual machine's object model—representation of runtime
//!   entities (objects, classes, methods, lookup tables). Objects are owned by
//!   the VM with inter-object references using handles/IDs rather than direct
//!   pointers, enabling future garbage collection.
//!
//! - **[`gc`]**: Garbage collection infrastructure. Currently a placeholder for
//!   future memory management. Will integrate with `vm` to reclaim unreachable
//!   objects.
//!
//! - **[`eval`]**: The evaluator that brings everything together. Walks the AST
//!   and executes it using the VM's object model—dispatching messages, looking
//!   up methods, and managing execution state.
//!
//! ### Data Flow
//!
//! 1. Source text is loaded into the **source manager**, which assigns file IDs
//! 2. The **lexer** scans the source, producing tokens with location info
//! 3. The **parser** consumes tokens and builds an AST
//! 4. The **evaluator** walks the AST, interacting with the **VM** to:
//!    - Create and manipulate objects
//!    - Send messages and dispatch methods
//!    - Manage the call stack and local variables
//! 5. **Diagnostics** are emitted at any stage when errors are encountered
//!
//! ### Design Principles
//!
//! - **Location tracking everywhere**: Every construct carries source location
//!   info so errors can point precisely to the problematic code.
//!
//! - **ID-based references**: Large structures (source files, runtime objects)
//!   are owned centrally with lightweight IDs used for references, avoiding
//!   complex lifetime management.
//!
//! - **Separation of concerns**: Each module has a focused responsibility,
//!   making the codebase easier to understand and modify.

pub mod ast;
pub mod diagnostics;
pub mod eval;
pub mod gc;
pub mod lexeme;
pub mod lexer;
pub mod parser;
pub mod source;
pub mod vm;

/// Returns the version string for the interpreter.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.1.0");
    }
}
