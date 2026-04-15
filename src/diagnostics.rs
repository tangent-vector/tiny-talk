//! # Diagnostics
//!
//! Unified error and warning reporting for all stages of interpretation.
//!
//! ## Responsibilities
//!
//! - **Diagnostic creation**: Provide a structured way to create error and warning
//!   messages with severity levels, error codes, and associated source locations.
//!
//! - **Message formatting**: Format diagnostics for human consumption, including
//!   source code snippets, location information, and helpful annotations.
//!
//! - **Severity levels**: Distinguish between errors (which prevent execution),
//!   warnings (which indicate potential problems), and notes (which provide
//!   additional context).
//!
//! - **Diagnostic collection**: Accumulate multiple diagnostics during processing
//!   to report all problems at once rather than stopping at the first error.
//!
//! - **Output flexibility**: Support different output formats and destinations
//!   (terminal with colors, plain text, JSON for tooling integration).
//!
//! ## Dependencies and Relationships
//!
//! This module depends on:
//! - `source`: to resolve locations and extract source snippets for display
//!
//! This module is used by:
//! - `lexer`: to report lexical errors (invalid characters, unterminated strings)
//! - `parser`: to report syntax errors (unexpected tokens, malformed constructs)
//! - `eval`: to report runtime errors (message not understood, type mismatches)
//! - The CLI: to display collected diagnostics to the user
//!
//! ## Architectural Approach
//!
//! ### Structured Diagnostics
//!
//! Each diagnostic is a structured object containing:
//! - A severity level (error, warning, note)
//! - A primary message describing the problem
//! - A primary source span indicating where the problem occurred
//! - Optional secondary spans with labels (for "see also" references)
//! - Optional fix suggestions
//!
//! This structure enables rich IDE integration and consistent formatting.
//!
//! ### Diagnostic Sink Pattern
//!
//! Rather than returning diagnostics directly, modules emit them to a **diagnostic
//! sink**—an abstraction that collects or immediately displays diagnostics. This
//! allows:
//! - Batch collection for later display
//! - Immediate streaming output
//! - Counting errors to decide whether to proceed
//! - Testing by capturing diagnostics for assertions
//!
//! ### Error Recovery
//!
//! The diagnostic system supports error recovery by allowing processing to continue
//! after emitting an error. The sink tracks whether any errors were emitted, letting
//! callers decide whether to proceed with potentially invalid results.
//!
//! ### Rendered Output
//!
//! Diagnostic rendering is separate from diagnostic creation. A renderer takes
//! diagnostics and a source manager, producing formatted output. This separation
//! allows multiple output formats without changing how diagnostics are created.
//!
//! The default renderer produces output similar to rustc's format:
//!
//! ```text
//! error: unexpected token
//!   --> example.tt:3:15
//!    |
//!  3 |     receiver message: argument
//!    |               ^^^^^^^ expected identifier
//! ```

// TODO: Implement diagnostic types and rendering
