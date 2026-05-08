//! # Lexemes
//!
//! Defines the vocabulary of tiny-talk: tokens and trivia.
//!
//! ## Responsibilities
//!
//! - **Token representation**: Define the set of semantically meaningful lexemes
//!   (identifiers, keywords, operators, literals, punctuation) that the parser
//!   operates on.
//!
//! - **Trivia representation**: Define lexemes that don't affect program semantics
//!   but are present in source text (whitespace, comments). These are captured for
//!   potential use in formatting tools but filtered out before parsing.
//!
//! - **Token classification**: Categorize tokens by their role (keyword vs identifier,
//!   binary operator vs special character) to simplify parser logic.
//!
//! - **Location attachment**: Every lexeme carries its source span, enabling precise
//!   error reporting and source-map generation.
//!
//! - **Literal values**: Store the semantic value of literal tokens (the actual
//!   integer value, the unescaped string content) alongside the raw source text.
//!
//! ## Dependencies and Relationships
//!
//! This module depends on:
//! - `source`: for span types to track lexeme locations
//!
//! This module is used by:
//! - `lexer`: produces lexemes from source text
//! - `parser`: consumes tokens to build the AST
//! - `ast`: may embed tokens for precise location tracking
//!
//! ## Architectural Approach
//!
//! ### Token vs Trivia Separation
//!
//! The lexer produces both tokens and trivia, but the parser only sees tokens. This
//! is handled in two ways:
//!
//! 1. **Filtering**: The lexer can produce a token-only stream for parsing
//! 2. **Attachment**: Trivia can be attached to adjacent tokens for tools that need
//!    to preserve formatting (pretty-printers, refactoring tools)
//!
//! For the initial tree-walking interpreter, we use simple filtering—trivia is
//! discarded after lexing.
//!
//! ### Newline Significance
//!
//! In classic Smalltalk, newlines are generally not significant—statements are
//! separated by periods. However, some Smalltalk variants use newlines in specific
//! contexts. The lexeme module defines newlines as trivia by default, but this can
//! be reconsidered based on the specific language rules we adopt.
//!
//! ### Token Kinds
//!
//! Tokens are categorized by kind:
//!
//! - **Identifiers**: Names that start with a letter (receiver, variable names)
//! - **Keywords**: Identifiers followed by a colon (message selectors with arguments)
//! - **Binary selectors**: Operator sequences like `+`, `->`, `==`
//! - **Literals**: Numbers, strings, symbols, characters
//! - **Punctuation**: Structural characters like `(`, `)`, `[`, `]`, `.`, `|`
//! - **Special**: Reserved tokens like `^` (return), `:=` (assignment)
//!
//! ### Smalltalk Lexical Conventions
//!
//! Some Smalltalk-specific lexical elements to handle:
//!
//! - **Symbol literals**: `#symbol` or `#'symbol with spaces'`
//! - **Character literals**: `$a` for the character 'a'
//! - **Block arguments**: `[:arg | ...]` syntax
//! - **Cascades**: The `;` for message cascades
//! - **Comments**: `"double-quoted comments"` (note: this differs from strings!)
//!
//! ### Efficient Storage
//!
//! Lexemes appear in large quantities, so storage efficiency matters. Common
//! approaches include:
//!
//! - Interning identifier strings
//! - Storing spans as compact (start, length) pairs
//! - Using a flat enum for token kinds to enable efficient pattern matching

use crate::source::Span;

// ============================================================================
// Trivia
// ============================================================================

/// Kinds of trivia (non-semantic lexemes).
///
/// Trivia includes whitespace and comments - elements that appear in source
/// text but don't affect the program's meaning. They are captured during
/// lexing for potential use by formatting tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriviaKind {
    /// Horizontal whitespace: spaces and tabs.
    Whitespace,
    /// Newline characters (\n, \r\n, or \r).
    Newline,
    /// A double-quoted comment: `"like this"`.
    ///
    /// Note: Smalltalk uses double quotes for comments, not strings!
    Comment,
}

/// A trivia element with its source location.
///
/// Trivia are captured during lexing but typically filtered out before
/// parsing. They can be attached to adjacent tokens for tools that need
/// to preserve formatting information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trivia {
    /// What kind of trivia this is.
    pub kind: TriviaKind,
    /// The source span covering this trivia.
    pub span: Span,
}

impl Trivia {
    /// Creates a new trivia element.
    pub fn new(kind: TriviaKind, span: Span) -> Self {
        Self { kind, span }
    }
}

// ============================================================================
// Token Kinds
// ============================================================================

/// All possible token kinds in tiny-talk.
///
/// This is a flat enum for efficient pattern matching. Each variant represents
/// a distinct lexical element that the parser may encounter.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // -------------------------------------------------------------------------
    // Identifiers and Keywords
    // -------------------------------------------------------------------------
    /// An identifier: a name starting with a letter.
    ///
    /// Examples: `x`, `receiver`, `self`, `super`, `nil`, `true`, `false`
    ///
    /// Note: Smalltalk doesn't have reserved words in the traditional sense.
    /// `self`, `super`, `nil`, `true`, `false` are pseudo-variables that
    /// look like identifiers but have special meaning.
    Identifier(String),

    /// A keyword selector: an identifier followed by a colon.
    ///
    /// Examples: `at:`, `put:`, `ifTrue:`, `ifFalse:`
    ///
    /// Multi-keyword messages are built from multiple keyword tokens:
    /// `at:put:` is lexed as two keywords: `at:` and `put:`.
    Keyword(String),

    // -------------------------------------------------------------------------
    // Binary Selectors
    // -------------------------------------------------------------------------
    /// A binary selector: one or more operator characters.
    ///
    /// Examples: `+`, `-`, `*`, `/`, `<`, `>`, `=`, `~`, `@`, `%`, `|`, `&`
    ///           `->`, `==`, `<=`, `>=`, `~~`
    ///
    /// Binary selectors are used for infix operators in binary messages.
    BinarySelector(String),

    // -------------------------------------------------------------------------
    // Literals
    // -------------------------------------------------------------------------
    /// An integer literal.
    ///
    /// Examples: `42`, `0`, `-17`, `16rFF` (hex), `2r1010` (binary)
    ///
    /// The stored value is the parsed integer. Radix prefixes are handled
    /// during lexing.
    Integer(i64),

    /// A floating-point literal.
    ///
    /// Examples: `3.14`, `1.0e10`, `2.5e-3`
    Float(f64),

    /// A string literal (single-quoted).
    ///
    /// Examples: `'hello'`, `'it''s'` (escaped single quote)
    ///
    /// The stored value is the unescaped string content.
    String(String),

    /// A symbol literal.
    ///
    /// Examples: `#symbol`, `#'symbol with spaces'`, `#+`
    ///
    /// Symbols are interned strings used as identifiers/keys.
    Symbol(String),

    /// A character literal.
    ///
    /// Examples: `$a`, `$\n`, `$ ` (space character)
    ///
    /// The stored value is the character.
    Character(char),

    /// An invalid or otherwise unrecognized token.
    ///
    /// The lexer emits this after reporting a diagnostic so later stages can
    /// keep processing the rest of the token stream.
    Error(String),

    // -------------------------------------------------------------------------
    // Punctuation
    // -------------------------------------------------------------------------
    /// Opening parenthesis `(`.
    LeftParen,
    /// Closing parenthesis `)`.
    RightParen,
    /// Opening bracket `[` (block start).
    LeftBracket,
    /// Closing bracket `]` (block end).
    RightBracket,
    /// Period `.` (statement separator).
    Period,
    /// Vertical bar `|` (temporaries delimiter, block argument separator).
    Pipe,
    /// Semicolon `;` (cascade separator).
    Semicolon,
    /// Colon `:` (used in block arguments like `[:x | ...]`).
    Colon,
    /// Hash/pound `#` (symbol prefix, array literal prefix).
    Hash,

    // -------------------------------------------------------------------------
    // Special
    // -------------------------------------------------------------------------
    /// Caret `^` (return operator).
    Caret,
    /// Assignment operator `:=`.
    Assign,

    // -------------------------------------------------------------------------
    // End of file
    // -------------------------------------------------------------------------
    /// End of input marker.
    ///
    /// This is emitted once at the end of the token stream.
    Eof,
}

impl TokenKind {
    /// Returns `true` if this is an identifier token.
    pub fn is_identifier(&self) -> bool {
        matches!(self, TokenKind::Identifier(_))
    }

    /// Returns `true` if this is a keyword token.
    pub fn is_keyword(&self) -> bool {
        matches!(self, TokenKind::Keyword(_))
    }

    /// Returns `true` if this is a binary selector token.
    pub fn is_binary_selector(&self) -> bool {
        matches!(self, TokenKind::BinarySelector(_))
    }

    /// Returns `true` if this is a literal token (integer, float, string, symbol, or character).
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            TokenKind::Integer(_)
                | TokenKind::Float(_)
                | TokenKind::String(_)
                | TokenKind::Symbol(_)
                | TokenKind::Character(_)
        )
    }

    /// Returns `true` if this is a punctuation token.
    pub fn is_punctuation(&self) -> bool {
        matches!(
            self,
            TokenKind::LeftParen
                | TokenKind::RightParen
                | TokenKind::LeftBracket
                | TokenKind::RightBracket
                | TokenKind::Period
                | TokenKind::Pipe
                | TokenKind::Semicolon
                | TokenKind::Colon
                | TokenKind::Hash
        )
    }

    /// Returns the string value if this is an identifier, keyword, binary selector, string, or symbol.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            TokenKind::Identifier(s)
            | TokenKind::Keyword(s)
            | TokenKind::BinarySelector(s)
            | TokenKind::String(s)
            | TokenKind::Symbol(s)
            | TokenKind::Error(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a human-readable description of this token kind for diagnostics.
    pub fn description(&self) -> &'static str {
        match self {
            TokenKind::Identifier(_) => "identifier",
            TokenKind::Keyword(_) => "keyword",
            TokenKind::BinarySelector(_) => "binary selector",
            TokenKind::Integer(_) => "integer",
            TokenKind::Float(_) => "float",
            TokenKind::String(_) => "string",
            TokenKind::Symbol(_) => "symbol",
            TokenKind::Character(_) => "character",
            TokenKind::Error(_) => "error",
            TokenKind::LeftParen => "'('",
            TokenKind::RightParen => "')'",
            TokenKind::LeftBracket => "'['",
            TokenKind::RightBracket => "']'",
            TokenKind::Period => "'.'",
            TokenKind::Pipe => "'|'",
            TokenKind::Semicolon => "';'",
            TokenKind::Colon => "':'",
            TokenKind::Hash => "'#'",
            TokenKind::Caret => "'^'",
            TokenKind::Assign => "':='",
            TokenKind::Eof => "end of file",
        }
    }
}

// ============================================================================
// Token
// ============================================================================

/// A token with its source location.
///
/// Tokens are the semantically meaningful lexemes that the parser operates on.
/// Each token carries:
/// - Its kind (what type of token it is, possibly with a value)
/// - Its source span (where in the source code it came from)
///
/// # Examples
///
/// ```ignore
/// let token = Token::new(TokenKind::Identifier("x".into()), span);
/// assert!(token.kind.is_identifier());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// What kind of token this is.
    pub kind: TokenKind,
    /// The source span covering this token.
    pub span: Span,
}

impl Token {
    /// Creates a new token.
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Creates a synthetic EOF token with no real source location.
    pub fn eof() -> Self {
        Self {
            kind: TokenKind::Eof,
            span: Span::synthetic(),
        }
    }

    /// Returns `true` if this is an EOF token.
    pub fn is_eof(&self) -> bool {
        matches!(self.kind, TokenKind::Eof)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::SourceId;

    fn test_span() -> Span {
        Span::new(SourceId::SYNTHETIC, 0, 1)
    }

    #[test]
    fn test_trivia_creation() {
        let span = test_span();
        let trivia = Trivia::new(TriviaKind::Whitespace, span);
        assert_eq!(trivia.kind, TriviaKind::Whitespace);
    }

    #[test]
    fn test_token_kind_classification() {
        assert!(TokenKind::Identifier("x".into()).is_identifier());
        assert!(TokenKind::Keyword("at:".into()).is_keyword());
        assert!(TokenKind::BinarySelector("+".into()).is_binary_selector());
        assert!(TokenKind::Integer(42).is_literal());
        assert!(TokenKind::Float(1.5).is_literal());
        assert!(TokenKind::String("hello".into()).is_literal());
        assert!(TokenKind::Symbol("sym".into()).is_literal());
        assert!(TokenKind::Character('a').is_literal());
        assert!(TokenKind::LeftParen.is_punctuation());
        assert!(TokenKind::Period.is_punctuation());
    }

    #[test]
    fn test_token_kind_as_str() {
        assert_eq!(TokenKind::Identifier("foo".into()).as_str(), Some("foo"));
        assert_eq!(TokenKind::Keyword("at:".into()).as_str(), Some("at:"));
        assert_eq!(TokenKind::BinarySelector("+".into()).as_str(), Some("+"));
        assert_eq!(TokenKind::String("hello".into()).as_str(), Some("hello"));
        assert_eq!(TokenKind::Symbol("sym".into()).as_str(), Some("sym"));
        assert_eq!(TokenKind::Error("bad".into()).as_str(), Some("bad"));
        assert_eq!(TokenKind::Integer(42).as_str(), None);
        assert_eq!(TokenKind::LeftParen.as_str(), None);
    }

    #[test]
    fn test_token_kind_description() {
        assert_eq!(
            TokenKind::Identifier("x".into()).description(),
            "identifier"
        );
        assert_eq!(TokenKind::LeftParen.description(), "'('");
        assert_eq!(TokenKind::Assign.description(), "':='");
        assert_eq!(TokenKind::Eof.description(), "end of file");
    }

    #[test]
    fn test_token_creation() {
        let span = test_span();
        let token = Token::new(TokenKind::Identifier("x".into()), span);
        assert!(token.kind.is_identifier());
        assert!(!token.is_eof());
    }

    #[test]
    fn test_eof_token() {
        let token = Token::eof();
        assert!(token.is_eof());
        assert_eq!(token.kind, TokenKind::Eof);
    }
}
