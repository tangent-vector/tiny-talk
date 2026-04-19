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

use std::path::{Path, PathBuf};

// ============================================================================
// Source Identifiers
// ============================================================================

/// A lightweight, copyable identifier for a source file.
///
/// `SourceId` is used throughout the codebase to refer to source files without
/// holding references to the actual content. The ID can be resolved to file
/// metadata and content through the [`SourceManager`].
///
/// # Examples
///
/// ```ignore
/// let mut manager = SourceManager::new();
/// let id = manager.add_file("example.tt", "x := 42.");
/// assert_eq!(manager.name(id), "example.tt");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceId(u32);

impl SourceId {
    /// A placeholder source ID for synthetic/generated code with no real source.
    ///
    /// Use this sparingly—most code should have real source locations.
    pub const SYNTHETIC: SourceId = SourceId(u32::MAX);
}

// ============================================================================
// Location and Span
// ============================================================================

/// A human-readable position in source code: 1-based line and column numbers.
///
/// This is the format users expect in error messages: "file.tt:10:5" means
/// line 10, column 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCol {
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number (in UTF-8 bytes from line start).
    pub column: u32,
}

/// A contiguous range of source text within a single file.
///
/// Spans are used to track the source location of tokens, AST nodes, and other
/// constructs. They enable precise error reporting and source-map generation.
///
/// # Representation
///
/// A span stores:
/// - The source file ID
/// - A byte offset where the span starts
/// - The length in bytes
///
/// This compact representation avoids redundantly storing the file ID twice
/// (once for start, once for end).
///
/// # Examples
///
/// ```ignore
/// // Create a span covering bytes 10-15 in a file
/// let span = Span::new(source_id, 10, 5);
/// assert_eq!(span.start(), 10);
/// assert_eq!(span.end(), 15);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    source: SourceId,
    start: u32,
    len: u32,
}

impl Span {
    /// Creates a new span.
    ///
    /// # Arguments
    ///
    /// * `source` - The source file this span belongs to
    /// * `start` - Byte offset where the span begins
    /// * `len` - Length of the span in bytes
    pub fn new(source: SourceId, start: u32, len: u32) -> Self {
        Self { source, start, len }
    }

    /// Creates a synthetic/placeholder span for generated code.
    pub fn synthetic() -> Self {
        Self {
            source: SourceId::SYNTHETIC,
            start: 0,
            len: 0,
        }
    }

    /// Returns the source file ID.
    pub fn source(&self) -> SourceId {
        self.source
    }

    /// Returns the starting byte offset.
    pub fn start(&self) -> u32 {
        self.start
    }

    /// Returns the ending byte offset (exclusive).
    pub fn end(&self) -> u32 {
        self.start + self.len
    }

    /// Returns the length in bytes.
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Returns true if this span has zero length.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Combines two spans into one that covers both.
    ///
    /// The spans must be from the same source file. The result spans from
    /// the start of the earlier span to the end of the later span.
    ///
    /// # Panics
    ///
    /// Panics if the spans are from different source files.
    pub fn merge(&self, other: &Span) -> Span {
        todo!("Span::merge")
    }
}

// ============================================================================
// Source File Storage
// ============================================================================

/// Metadata and content for a single source file.
///
/// This is an internal type managed by [`SourceManager`]. It stores:
/// - The file name/path
/// - The source text content
/// - A line index for efficient line/column lookups
struct SourceFile {
    /// The name or path of the source file (for display in diagnostics).
    name: String,
    /// The full source text content.
    content: String,
    /// Byte offsets where each line begins (computed lazily or eagerly).
    line_starts: Vec<u32>,
}

impl SourceFile {
    /// Creates a new source file with the given name and content.
    fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        todo!("SourceFile::new")
    }

    /// Computes the line/column position for a byte offset.
    fn line_col(&self, offset: u32) -> LineCol {
        todo!("SourceFile::line_col")
    }

    /// Returns the byte offset where a given line starts (0-based line index).
    fn line_start(&self, line_index: usize) -> Option<u32> {
        todo!("SourceFile::line_start")
    }

    /// Returns the text content of a given line (0-based line index).
    fn line_text(&self, line_index: usize) -> Option<&str> {
        todo!("SourceFile::line_text")
    }
}

// ============================================================================
// Source Manager
// ============================================================================

/// Central registry and owner of all source files.
///
/// The `SourceManager` is the single source of truth for source text in the
/// interpreter. Other modules reference source locations using [`SourceId`]
/// and [`Span`], resolving them through the manager when needed.
///
/// # Examples
///
/// ```ignore
/// let mut manager = SourceManager::new();
///
/// // Add a source file
/// let id = manager.add_file("hello.tt", "'Hello, World!' print.");
///
/// // Query file metadata
/// assert_eq!(manager.name(id), "hello.tt");
///
/// // Get line/column for a byte offset
/// let pos = manager.line_col(id, 0);
/// assert_eq!(pos, LineCol { line: 1, column: 1 });
/// ```
pub struct SourceManager {
    files: Vec<SourceFile>,
}

impl SourceManager {
    /// Creates a new, empty source manager.
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Adds a source file with the given name and content.
    ///
    /// Returns a [`SourceId`] that can be used to refer to this file.
    pub fn add_file(&mut self, name: impl Into<String>, content: impl Into<String>) -> SourceId {
        todo!("SourceManager::add_file")
    }

    /// Adds a source file by reading from a filesystem path.
    ///
    /// The file's display name will be the path as provided.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn add_file_from_path(&mut self, path: impl AsRef<Path>) -> std::io::Result<SourceId> {
        todo!("SourceManager::add_file_from_path")
    }

    /// Returns the name/path of a source file.
    ///
    /// # Panics
    ///
    /// Panics if the source ID is invalid.
    pub fn name(&self, id: SourceId) -> &str {
        todo!("SourceManager::name")
    }

    /// Returns the full content of a source file.
    ///
    /// # Panics
    ///
    /// Panics if the source ID is invalid.
    pub fn content(&self, id: SourceId) -> &str {
        todo!("SourceManager::content")
    }

    /// Computes the line/column position for a byte offset in a file.
    ///
    /// # Panics
    ///
    /// Panics if the source ID is invalid or the offset is out of bounds.
    pub fn line_col(&self, id: SourceId, offset: u32) -> LineCol {
        todo!("SourceManager::line_col")
    }

    /// Returns the line/column for the start of a span.
    pub fn span_start_line_col(&self, span: Span) -> LineCol {
        self.line_col(span.source(), span.start())
    }

    /// Returns the line/column for the end of a span.
    pub fn span_end_line_col(&self, span: Span) -> LineCol {
        self.line_col(span.source(), span.end())
    }

    /// Returns the text content of a specific line (1-based line number).
    ///
    /// Returns `None` if the line number is out of bounds.
    pub fn line_text(&self, id: SourceId, line: u32) -> Option<&str> {
        todo!("SourceManager::line_text")
    }

    /// Extracts the source text covered by a span.
    ///
    /// # Panics
    ///
    /// Panics if the span's source ID is invalid or the span is out of bounds.
    pub fn span_text(&self, span: Span) -> &str {
        todo!("SourceManager::span_text")
    }

    /// Returns the number of lines in a source file.
    pub fn line_count(&self, id: SourceId) -> usize {
        todo!("SourceManager::line_count")
    }
}

impl Default for SourceManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_id_synthetic() {
        // Synthetic ID should be usable without panicking
        let span = Span::synthetic();
        assert_eq!(span.source(), SourceId::SYNTHETIC);
    }

    #[test]
    fn test_span_basic() {
        let span = Span::new(SourceId(0), 10, 5);
        assert_eq!(span.start(), 10);
        assert_eq!(span.end(), 15);
        assert_eq!(span.len(), 5);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_span_empty() {
        let span = Span::new(SourceId(0), 10, 0);
        assert!(span.is_empty());
    }

    // Additional tests will be added during implementation
}
