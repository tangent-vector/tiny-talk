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

use crate::source::{SourceManager, Span};
use std::io::Write;

// ============================================================================
// Severity
// ============================================================================

/// The severity level of a diagnostic.
///
/// Severity determines how the diagnostic is displayed and whether it
/// prevents further processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// An error that prevents successful completion.
    ///
    /// Errors indicate problems that must be fixed before the program can run.
    /// Examples: syntax errors, undefined variables, type mismatches.
    Error,

    /// A warning about potentially problematic code.
    ///
    /// Warnings don't prevent execution but indicate code that may not behave
    /// as intended. Examples: unused variables, deprecated features.
    Warning,

    /// Additional context or information.
    ///
    /// Notes are attached to other diagnostics to provide more detail.
    /// They're not standalone issues. Example: "see also: definition here"
    Note,
}

impl Severity {
    /// Returns a lowercase string representation for display.
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        }
    }
}

// ============================================================================
// Labels
// ============================================================================

/// A labeled source span within a diagnostic.
///
/// Labels annotate specific regions of source code with explanatory text.
/// A diagnostic has one primary label and zero or more secondary labels.
#[derive(Debug, Clone)]
pub struct Label {
    /// The source span this label annotates.
    span: Span,
    /// The explanatory message for this span.
    message: String,
    /// Whether this is the primary label (determines styling).
    is_primary: bool,
}

impl Label {
    /// Creates a primary label (the main location of the diagnostic).
    ///
    /// Primary labels are displayed with emphasis (e.g., `^^^` underline).
    pub fn primary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            is_primary: true,
        }
    }

    /// Creates a secondary label (additional context).
    ///
    /// Secondary labels are displayed with less emphasis (e.g., `---` underline).
    pub fn secondary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            is_primary: false,
        }
    }

    /// Returns the span this label annotates.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Returns the label's message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns whether this is a primary label.
    pub fn is_primary(&self) -> bool {
        self.is_primary
    }
}

// ============================================================================
// Diagnostic
// ============================================================================

/// A single diagnostic message (error, warning, or note).
///
/// Diagnostics are the primary way the interpreter communicates problems
/// to users. Each diagnostic includes:
///
/// - A severity level (error/warning/note)
/// - A human-readable message
/// - One or more labeled source locations
/// - Optional help text with suggestions
///
/// # Examples
///
/// ```ignore
/// let diag = Diagnostic::error("undefined variable")
///     .with_label(Label::primary(span, "not found in this scope"))
///     .with_help("did you mean `count`?");
/// ```
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The severity of this diagnostic.
    severity: Severity,
    /// The main message describing the problem.
    message: String,
    /// Labeled source locations.
    labels: Vec<Label>,
    /// Optional help text with suggestions.
    help: Option<String>,
}

impl Diagnostic {
    /// Creates a new error diagnostic.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            labels: Vec::new(),
            help: None,
        }
    }

    /// Creates a new warning diagnostic.
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            labels: Vec::new(),
            help: None,
        }
    }

    /// Creates a new note diagnostic.
    pub fn note(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Note,
            message: message.into(),
            labels: Vec::new(),
            help: None,
        }
    }

    /// Creates a diagnostic with the given severity.
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            labels: Vec::new(),
            help: None,
        }
    }

    /// Adds a label to this diagnostic.
    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    /// Adds help text to this diagnostic.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Returns the severity of this diagnostic.
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the main message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the labels attached to this diagnostic.
    pub fn labels(&self) -> &[Label] {
        &self.labels
    }

    /// Returns the primary label, if any.
    pub fn primary_label(&self) -> Option<&Label> {
        self.labels.iter().find(|l| l.is_primary)
    }

    /// Returns the help text, if any.
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    /// Returns true if this is an error diagnostic.
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

// ============================================================================
// Diagnostic Sink
// ============================================================================

/// A receiver for diagnostic messages.
///
/// The `DiagnosticSink` trait abstracts how diagnostics are handled. This
/// enables:
///
/// - Collecting diagnostics for batch processing
/// - Streaming diagnostics to output immediately
/// - Testing by capturing diagnostics for assertions
/// - Counting errors to decide whether to abort
///
/// Implementations should track error counts to support error recovery.
pub trait DiagnosticSink {
    /// Emits a diagnostic.
    fn emit(&mut self, diagnostic: Diagnostic);

    /// Returns the number of errors emitted so far.
    fn error_count(&self) -> usize;

    /// Returns true if any errors have been emitted.
    fn has_errors(&self) -> bool {
        self.error_count() > 0
    }
}

// ============================================================================
// Diagnostic Collector
// ============================================================================

/// A diagnostic sink that collects diagnostics for later processing.
///
/// Use this when you want to:
/// - Accumulate all diagnostics before displaying them
/// - Run multiple passes and combine their diagnostics
/// - Test code by inspecting the collected diagnostics
///
/// # Examples
///
/// ```ignore
/// let mut collector = DiagnosticCollector::new();
/// // ... run lexer, parser, etc., emitting to collector ...
/// if collector.has_errors() {
///     for diag in collector.diagnostics() {
///         renderer.render(diag, &sources);
///     }
/// }
/// ```
#[derive(Debug, Default)]
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
    error_count: usize,
}

impl DiagnosticCollector {
    /// Creates a new empty collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns all collected diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Consumes the collector and returns the diagnostics.
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    /// Clears all collected diagnostics and resets error count.
    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.error_count = 0;
    }
}

impl DiagnosticSink for DiagnosticCollector {
    fn emit(&mut self, diagnostic: Diagnostic) {
        if diagnostic.is_error() {
            self.error_count += 1;
        }
        self.diagnostics.push(diagnostic);
    }

    fn error_count(&self) -> usize {
        self.error_count
    }
}

// ============================================================================
// Diagnostic Renderer
// ============================================================================

/// Renders diagnostics to a text output.
///
/// The renderer formats diagnostics with source context, producing output
/// similar to rustc:
///
/// ```text
/// error: unexpected token
///   --> example.tt:3:15
///    |
///  3 |     receiver message: argument
///    |               ^^^^^^^ expected identifier
/// ```
///
/// # Examples
///
/// ```ignore
/// let renderer = DiagnosticRenderer::new();
/// renderer.render(&diagnostic, &sources, &mut std::io::stderr())?;
/// ```
pub struct DiagnosticRenderer {
    /// Whether to use colors in output (for terminal display).
    use_colors: bool,
}

impl DiagnosticRenderer {
    /// Creates a new renderer with default settings.
    pub fn new() -> Self {
        Self { use_colors: false }
    }

    /// Creates a renderer with color output enabled.
    pub fn with_colors() -> Self {
        Self { use_colors: true }
    }

    /// Enables or disables color output.
    pub fn set_colors(&mut self, enabled: bool) {
        self.use_colors = enabled;
    }

    /// Returns whether color output is enabled.
    pub fn uses_colors(&self) -> bool {
        self.use_colors
    }

    /// Renders a single diagnostic to the given writer.
    ///
    /// The output includes:
    /// - Severity and message on the first line
    /// - File location (file:line:column)
    /// - Source code snippet with underlined spans
    /// - Help text if present
    pub fn render(
        &self,
        diagnostic: &Diagnostic,
        sources: &SourceManager,
        writer: &mut dyn Write,
    ) -> std::io::Result<()> {
        todo!("DiagnosticRenderer::render")
    }

    /// Renders all diagnostics from a collector.
    pub fn render_all(
        &self,
        diagnostics: &[Diagnostic],
        sources: &SourceManager,
        writer: &mut dyn Write,
    ) -> std::io::Result<()> {
        for diag in diagnostics {
            self.render(diag, sources, writer)?;
            writeln!(writer)?;
        }
        Ok(())
    }
}

impl Default for DiagnosticRenderer {
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
    use crate::source::SourceId;

    #[test]
    fn test_severity_ordering() {
        // Errors are most severe
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Note);
    }

    #[test]
    fn test_severity_as_str() {
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert_eq!(Severity::Note.as_str(), "note");
    }

    #[test]
    fn test_diagnostic_builder() {
        let span = Span::new(SourceId::SYNTHETIC, 0, 5);
        let diag = Diagnostic::error("test error")
            .with_label(Label::primary(span, "here"))
            .with_help("try this instead");

        assert!(diag.is_error());
        assert_eq!(diag.message(), "test error");
        assert_eq!(diag.labels().len(), 1);
        assert!(diag.primary_label().is_some());
        assert_eq!(diag.help(), Some("try this instead"));
    }

    #[test]
    fn test_diagnostic_collector() {
        let mut collector = DiagnosticCollector::new();
        assert_eq!(collector.error_count(), 0);
        assert!(!collector.has_errors());

        collector.emit(Diagnostic::warning("a warning"));
        assert_eq!(collector.error_count(), 0);
        assert!(!collector.has_errors());

        collector.emit(Diagnostic::error("an error"));
        assert_eq!(collector.error_count(), 1);
        assert!(collector.has_errors());

        assert_eq!(collector.diagnostics().len(), 2);
    }

    #[test]
    fn test_label_primary_secondary() {
        let span = Span::new(SourceId::SYNTHETIC, 10, 5);

        let primary = Label::primary(span, "primary message");
        assert!(primary.is_primary());
        assert_eq!(primary.message(), "primary message");

        let secondary = Label::secondary(span, "secondary message");
        assert!(!secondary.is_primary());
        assert_eq!(secondary.message(), "secondary message");
    }
}
