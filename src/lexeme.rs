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

// TODO: Define token and trivia types
