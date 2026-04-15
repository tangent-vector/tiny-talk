//! # Source Management
//!
//! Manages source files and tracks locations within them.
//!
//! ## Responsibilities
//!
//! - **Source file storage**: Load and store the contents of source files, providing
//!   the raw text that all subsequent processing stages operate on.
//!
//! - **File identification**: Assign unique identifiers to source files so that other
//!   modules can refer to files without holding the actual content.
//!
//! - **Location tracking**: Represent positions within source files as (line, column)
//!   coordinates, with efficient translation between byte offsets and human-readable
//!   positions.
//!
//! - **Span representation**: Represent contiguous ranges of source text (e.g., the
//!   extent of a token or AST node) for error reporting and tooling.
//!
//! - **Source snippets**: Extract fragments of source text for display in diagnostics,
//!   including the surrounding context lines.
//!
//! ## Dependencies and Relationships
//!
//! This module depends on:
//! - No other tiny-talk modules (this is a foundational module)
//!
//! This module is used by:
//! - `diagnostics`: to display source locations and code snippets in error messages
//! - `lexeme`: to attach location information to tokens
//! - `lexer`: to track position while scanning
//! - `ast`: to attach span information to syntax nodes
//! - `parser`: to track positions during parsing
//! - `eval`: to report runtime error locations
//!
//! ## Architectural Approach
//!
//! ### Centralized Ownership with Lightweight IDs
//!
//! Source files can be large, and many parts of the system need to refer to locations
//! within them. Rather than passing around string slices with complex lifetimes, we
//! use a **source manager** pattern:
//!
//! - A central manager owns all source file contents
//! - Each file is assigned a small, copyable file ID
//! - Locations are represented as (file ID, byte offset) pairs
//! - Client code uses IDs for storage and queries the manager when needed
//!
//! This approach simplifies lifetime management throughout the codebase and makes it
//! easy to serialize/deserialize location information.
//!
//! ### Efficient Line/Column Computation
//!
//! Byte offsets are efficient for internal use but users expect line/column numbers.
//! The source manager maintains a **line index** for each file—a sorted list of byte
//! offsets where each line begins. This enables O(log n) lookup from offset to line
//! number via binary search.
//!
//! ### Span Representation
//!
//! A span is simply a (start, end) pair of locations. For efficiency, spans within
//! a single file can use just byte offsets plus a file ID, avoiding redundant storage
//! of the file ID in both start and end positions.
//!
//! ### Lazy Loading Considerations
//!
//! For future IDE integration, the architecture allows for lazy loading of source
//! content—file IDs can be registered before content is loaded, with the actual text
//! fetched on demand.

// TODO: Implement source management types and functions
