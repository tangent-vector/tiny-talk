# Agent Guidelines for tiny-talk

This document provides guidance for agents working on the tiny-talk codebase.

## Building and Testing

tiny-talk is a standard Rust/Cargo project. Use these commands:

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the interpreter
cargo run

# Check for compile errors without full build
cargo check
```

## Module Architecture Documentation

Each module in tiny-talk should begin with a comprehensive documentation comment that explains the module's role in the overall architecture. This comment should be in Markdown format and include the following sections:

### Required Documentation Sections

1. **Module Purpose** (brief, 1-2 sentences)
   - What this module does at a high level

2. **Responsibilities**
   - Detailed breakdown of the services this module provides
   - Use bullet points for clarity

3. **Dependencies and Relationships**
   - Which other modules this module depends on
   - Which modules depend on this one
   - How data flows between modules

4. **Architectural Approach**
   - High-level design decisions
   - Key abstractions and patterns used
   - This should read like a whiteboard discussion, not API documentation
   - Avoid getting into specific type/function names at this level

### Example Structure

```rust
//! # Module Name
//!
//! Brief one-line description of the module's purpose.
//!
//! ## Responsibilities
//!
//! - First responsibility
//! - Second responsibility
//! - ...
//!
//! ## Dependencies and Relationships
//!
//! This module depends on:
//! - `other_module`: for X functionality
//!
//! This module is used by:
//! - `consumer_module`: to provide Y services
//!
//! ## Architectural Approach
//!
//! Description of the design approach, key patterns, and rationale...
```

## Code Style

- Follow standard Rust idioms and naming conventions
- Use `rustfmt` for formatting
- Prefer explicit error handling over panics in library code
- Document public APIs with doc comments

## Project Structure

The crate is organized into focused modules:

- `src/lib.rs` - Library root with top-level architecture documentation
- `src/source.rs` - Source file representation and location tracking
- `src/diagnostics.rs` - Error and warning reporting
- `src/lexeme.rs` - Token and trivia representation
- `src/lexer.rs` - Lexical analysis
- `src/ast.rs` - Abstract syntax tree
- `src/parser.rs` - Parsing from tokens to AST
- `src/vm.rs` - Runtime object representation
- `src/gc.rs` - Garbage collection (future)
- `src/eval.rs` - AST evaluation/interpretation
- `src/main.rs` - CLI entry point
