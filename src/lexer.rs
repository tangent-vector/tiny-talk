//! # Lexer
//!
//! Transforms source text into a sequence of tokens.
//!
//! ## Responsibilities
//!
//! - **Character scanning**: Read through source text character by character,
//!   recognizing the boundaries and content of lexemes.
//!
//! - **Token production**: Produce a stream of tokens (semantically meaningful
//!   lexemes) for the parser to consume.
//!
//! - **Trivia handling**: Recognize and skip over whitespace and comments,
//!   optionally preserving them for tools that need formatting information.
//!
//! - **Literal parsing**: Parse the content of literal tokens (numbers, strings)
//!   into their semantic values, handling escape sequences and numeric formats.
//!
//! - **Error recovery**: Handle invalid input gracefully, producing error tokens
//!   and continuing to lex the rest of the file rather than aborting.
//!
//! - **Location tracking**: Track the current position in the source file and
//!   attach accurate spans to each produced lexeme.
//!
//! ## Dependencies and Relationships
//!
//! This module depends on:
//! - `source`: for source file access and location tracking
//! - `lexeme`: for token type definitions
//! - `diagnostics`: for reporting lexical errors
//!
//! This module is used by:
//! - `parser`: consumes the token stream to build an AST
//! - The CLI: may lex files for syntax highlighting or validation
//!
//! ## Architectural Approach
//!
//! ### Batch Lexing
//!
//! For simplicity, the lexer processes an entire source file at once, producing
//! a complete vector of tokens. This avoids the complexity of incremental or
//! streaming lexer interfaces while being sufficient for a tree-walking interpreter.
//!
//! The typical flow is:
//! 1. Load source file into the source manager
//! 2. Call the lexer with the file ID
//! 3. Receive a vector of tokens (trivia filtered out)
//! 4. Pass tokens to the parser
//!
//! ### Scanning Strategy
//!
//! The lexer uses a simple **maximal munch** strategy: at each position, it tries
//! to match the longest possible token. This is implemented as a state machine
//! or a series of conditional checks based on the current character.
//!
//! Key decision points:
//! - Letter → identifier or keyword (if followed by `:`)
//! - Digit → number literal
//! - `"` → comment (in Smalltalk, double quotes are comments!)
//! - `'` → string literal
//! - `#` → symbol literal
//! - `$` → character literal
//! - Operator characters → binary selector
//!
//! ### Smalltalk Lexical Quirks
//!
//! Several Smalltalk conventions differ from mainstream languages:
//!
//! - **Comments in double quotes**: `"this is a comment"` — strings use single quotes
//! - **Keywords include the colon**: `at:put:` is a single selector, not three tokens
//! - **Identifiers can't have underscores** (in classic Smalltalk)
//! - **Numbers can have radix**: `16rFF` for hexadecimal
//! - **Negative numbers**: handled at the parser level (unary minus)
//!
//! ### Error Handling
//!
//! When the lexer encounters invalid input:
//! 1. Emit a diagnostic describing the problem
//! 2. Produce an "error" token spanning the problematic text
//! 3. Advance past the invalid input and continue lexing
//!
//! This allows the parser to see a complete token stream and potentially continue
//! its own error recovery.
//!
//! ### Lookahead
//!
//! Most token recognition requires only one character of lookahead. The lexer
//! maintains a **peek** capability to examine the next character without consuming
//! it. Multi-character lookahead is rarely needed but can be added for specific
//! cases (e.g., distinguishing `<-` from `<`).

// TODO: Implement lexer
