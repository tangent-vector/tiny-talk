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

use crate::diagnostics::{Diagnostic, DiagnosticSink, Label};
use crate::lexeme::{Token, TokenKind};
use crate::source::{SourceId, SourceManager, Span};

/// Lexes the source file identified by `source_id` into parser-ready tokens.
///
/// Whitespace and comments are skipped. Lexical problems are reported through
/// `diagnostics`, and an [`TokenKind::Error`] token is emitted for each
/// recoverable bad lexeme so downstream stages can continue.
pub fn lex(
    sources: &SourceManager,
    source_id: SourceId,
    diagnostics: &mut impl DiagnosticSink,
) -> Vec<Token> {
    Lexer::new(sources.content(source_id), source_id, diagnostics).lex_tokens()
}

/// Lexes an in-memory string using a synthetic source ID.
///
/// This convenience entry point is useful for tests and callers that do not
/// need source-manager-backed diagnostics. For user-facing files, prefer
/// [`lex`] so emitted spans point at real source text.
pub fn lex_synthetic(source: &str, diagnostics: &mut impl DiagnosticSink) -> Vec<Token> {
    Lexer::new(source, SourceId::SYNTHETIC, diagnostics).lex_tokens()
}

struct Lexer<'src, 'diag, D: DiagnosticSink> {
    source: &'src str,
    source_id: SourceId,
    offset: usize,
    diagnostics: &'diag mut D,
}

impl<'src, 'diag, D: DiagnosticSink> Lexer<'src, 'diag, D> {
    fn new(source: &'src str, source_id: SourceId, diagnostics: &'diag mut D) -> Self {
        Self {
            source,
            source_id,
            offset: 0,
            diagnostics,
        }
    }

    fn lex_tokens(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_trivia();
            if self.is_at_end() {
                break;
            }

            let start = self.offset;
            let ch = self.advance().expect("not at end");
            let kind = match ch {
                'a'..='z' | 'A'..='Z' => self.identifier_or_keyword(start),
                '0'..='9' => self.number(start),
                '\'' => self.string(start),
                '#' => self.symbol_or_hash(start),
                '$' => self.character(start),
                '(' => TokenKind::LeftParen,
                ')' => TokenKind::RightParen,
                '[' => TokenKind::LeftBracket,
                ']' => TokenKind::RightBracket,
                '.' => TokenKind::Period,
                ';' => TokenKind::Semicolon,
                '^' => TokenKind::Caret,
                ':' if self.match_char('=') => TokenKind::Assign,
                ':' => TokenKind::Colon,
                '|' => {
                    if self.peek().is_some_and(is_operator_continuation) {
                        self.binary_selector(start)
                    } else {
                        TokenKind::Pipe
                    }
                }
                c if is_binary_selector_start(c) => self.binary_selector(start),
                c => self.error_token(start, self.offset, format!("unexpected character `{c}`")),
            };
            tokens.push(Token::new(kind, self.span_from(start)));
        }

        tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(self.source_id, self.offset as u32, 0),
        ));
        tokens
    }

    fn skip_trivia(&mut self) {
        loop {
            let Some(ch) = self.peek() else {
                return;
            };

            match ch {
                ' ' | '\t' | '\u{000C}' => {
                    self.advance();
                }
                '\n' | '\r' => {
                    self.advance();
                    if ch == '\r' {
                        self.match_char('\n');
                    }
                }
                '"' => self.comment(),
                _ => return,
            }
        }
    }

    fn comment(&mut self) {
        let start = self.offset;
        self.advance();
        while let Some(ch) = self.peek() {
            self.advance();
            if ch == '"' {
                return;
            }
        }
        self.emit_error(start, self.offset, "unterminated comment");
    }

    fn identifier_or_keyword(&mut self, start: usize) -> TokenKind {
        while self.peek().is_some_and(is_identifier_continue) {
            self.advance();
        }

        if self.match_char(':') {
            TokenKind::Keyword(self.source[start..self.offset].to_owned())
        } else {
            TokenKind::Identifier(self.source[start..self.offset].to_owned())
        }
    }

    fn number(&mut self, start: usize) -> TokenKind {
        while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            self.advance();
        }

        if self.peek().is_some_and(|ch| ch == 'r' || ch == 'R') {
            let radix_text = &self.source[start..self.offset];
            let radix = match radix_text.parse::<u32>() {
                Ok(radix) if (2..=36).contains(&radix) => radix,
                _ => {
                    return self.error_token(
                        start,
                        self.offset,
                        format!("invalid radix `{radix_text}`; expected 2 through 36"),
                    )
                }
            };

            self.advance();
            let digits_start = self.offset;
            while self.peek().is_some_and(|ch| ch.is_ascii_alphanumeric()) {
                self.advance();
            }

            if digits_start == self.offset {
                return self.error_token(start, self.offset, "missing digits after radix prefix");
            }

            return match i64::from_str_radix(&self.source[digits_start..self.offset], radix) {
                Ok(value) => TokenKind::Integer(value),
                Err(_) => self.error_token(
                    start,
                    self.offset,
                    format!("invalid base-{radix} integer literal"),
                ),
            };
        }

        let mut is_float = false;
        if self.peek() == Some('.') && self.peek_next().is_some_and(|ch| ch.is_ascii_digit()) {
            is_float = true;
            self.advance();
            while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                self.advance();
            }
        }

        if self.peek().is_some_and(|ch| ch == 'e' || ch == 'E') {
            let exponent_marker = self.offset;
            self.advance();
            if self.peek().is_some_and(|ch| ch == '+' || ch == '-') {
                self.advance();
            }
            let exponent_digits = self.offset;
            while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                self.advance();
            }
            if exponent_digits == self.offset {
                return self.error_token(start, self.offset, "missing exponent digits");
            }
            if exponent_marker > start {
                is_float = true;
            }
        }

        let text = &self.source[start..self.offset];
        if is_float {
            match text.parse::<f64>() {
                Ok(value) => TokenKind::Float(value),
                Err(_) => self.error_token(start, self.offset, "invalid floating-point literal"),
            }
        } else {
            match text.parse::<i64>() {
                Ok(value) => TokenKind::Integer(value),
                Err(_) => self.error_token(start, self.offset, "invalid integer literal"),
            }
        }
    }

    fn string(&mut self, start: usize) -> TokenKind {
        let mut value = String::new();
        loop {
            let Some(ch) = self.advance() else {
                return self.error_token(start, self.offset, "unterminated string literal");
            };

            if ch == '\'' {
                if self.match_char('\'') {
                    value.push('\'');
                } else {
                    return TokenKind::String(value);
                }
            } else {
                value.push(ch);
            }
        }
    }

    fn symbol_or_hash(&mut self, start: usize) -> TokenKind {
        let Some(ch) = self.peek() else {
            return TokenKind::Hash;
        };

        match ch {
            '\'' => {
                self.advance();
                match self.string(self.offset - 1) {
                    TokenKind::String(value) => TokenKind::Symbol(value),
                    TokenKind::Error(_) => {
                        self.error_token(start, self.offset, "unterminated symbol literal")
                    }
                    _ => unreachable!(),
                }
            }
            'a'..='z' | 'A'..='Z' => {
                self.advance();
                while self.peek().is_some_and(is_identifier_continue) {
                    self.advance();
                }
                while self.match_char(':') {
                    while self.peek().is_some_and(is_identifier_continue) {
                        self.advance();
                    }
                }
                TokenKind::Symbol(self.source[start + 1..self.offset].to_owned())
            }
            c if is_binary_selector_start(c) => {
                self.advance();
                while self.peek().is_some_and(is_operator_continuation) {
                    self.advance();
                }
                TokenKind::Symbol(self.source[start + 1..self.offset].to_owned())
            }
            '(' => TokenKind::Hash,
            _ => TokenKind::Hash,
        }
    }

    fn character(&mut self, start: usize) -> TokenKind {
        let Some(ch) = self.advance() else {
            return self.error_token(start, self.offset, "missing character after `$`");
        };
        TokenKind::Character(ch)
    }

    fn binary_selector(&mut self, start: usize) -> TokenKind {
        while self.peek().is_some_and(is_operator_continuation) {
            self.advance();
        }
        TokenKind::BinarySelector(self.source[start..self.offset].to_owned())
    }

    fn error_token(&mut self, start: usize, end: usize, message: impl Into<String>) -> TokenKind {
        let message = message.into();
        self.emit_error(start, end, message.clone());
        TokenKind::Error(self.source[start..end].to_owned())
    }

    fn emit_error(&mut self, start: usize, end: usize, message: impl Into<String>) {
        let len = end.saturating_sub(start).max(1);
        let span = Span::new(self.source_id, start as u32, len as u32);
        self.diagnostics.emit(
            Diagnostic::error(message).with_label(Label::primary(span, "lexical error here")),
        );
    }

    fn span_from(&self, start: usize) -> Span {
        Span::new(self.source_id, start as u32, (self.offset - start) as u32)
    }

    fn is_at_end(&self) -> bool {
        self.offset >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.offset..].chars();
        chars.next()?;
        chars.next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
}

fn is_binary_selector_start(ch: char) -> bool {
    matches!(
        ch,
        '+' | '-' | '*' | '/' | '\\' | '~' | '<' | '>' | '=' | '@' | '%' | '&' | '?' | ',' | '!'
    )
}

fn is_operator_continuation(ch: char) -> bool {
    is_binary_selector_start(ch) || matches!(ch, '|' | '^')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{DiagnosticCollector, DiagnosticSink};

    fn kinds(source: &str) -> (Vec<TokenKind>, DiagnosticCollector) {
        let mut diagnostics = DiagnosticCollector::new();
        let tokens = lex_synthetic(source, &mut diagnostics)
            .into_iter()
            .map(|token| token.kind)
            .collect();
        (tokens, diagnostics)
    }

    #[test]
    fn lexes_identifiers_keywords_and_punctuation() {
        let (tokens, diagnostics) = kinds("receiver at: 1 put: 2. ^self");
        assert!(!diagnostics.has_errors());
        assert_eq!(
            tokens,
            vec![
                TokenKind::Identifier("receiver".into()),
                TokenKind::Keyword("at:".into()),
                TokenKind::Integer(1),
                TokenKind::Keyword("put:".into()),
                TokenKind::Integer(2),
                TokenKind::Period,
                TokenKind::Caret,
                TokenKind::Identifier("self".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_whitespace_and_smalltalk_comments() {
        let (tokens, diagnostics) = kinds("  foo \" ignored \"\n\tbar");
        assert!(!diagnostics.has_errors());
        assert_eq!(
            tokens,
            vec![
                TokenKind::Identifier("foo".into()),
                TokenKind::Identifier("bar".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_literals() {
        let (tokens, diagnostics) = kinds("42 2.5 1e3 16rFF 'it''s' #name #'two words' #+ $x");
        assert!(!diagnostics.has_errors());
        assert_eq!(
            tokens,
            vec![
                TokenKind::Integer(42),
                TokenKind::Float(2.5),
                TokenKind::Float(1000.0),
                TokenKind::Integer(255),
                TokenKind::String("it's".into()),
                TokenKind::Symbol("name".into()),
                TokenKind::Symbol("two words".into()),
                TokenKind::Symbol("+".into()),
                TokenKind::Character('x'),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_assignment_binary_selectors_and_delimiters() {
        let (tokens, diagnostics) = kinds("[:x | x := a <= b; yourself]");
        assert!(!diagnostics.has_errors());
        assert_eq!(
            tokens,
            vec![
                TokenKind::LeftBracket,
                TokenKind::Colon,
                TokenKind::Identifier("x".into()),
                TokenKind::Pipe,
                TokenKind::Identifier("x".into()),
                TokenKind::Assign,
                TokenKind::Identifier("a".into()),
                TokenKind::BinarySelector("<=".into()),
                TokenKind::Identifier("b".into()),
                TokenKind::Semicolon,
                TokenKind::Identifier("yourself".into()),
                TokenKind::RightBracket,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn leaves_array_literal_hash_as_punctuation() {
        let (tokens, diagnostics) = kinds("#(1 2)");
        assert!(!diagnostics.has_errors());
        assert_eq!(
            tokens,
            vec![
                TokenKind::Hash,
                TokenKind::LeftParen,
                TokenKind::Integer(1),
                TokenKind::Integer(2),
                TokenKind::RightParen,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn reports_errors_and_recovers() {
        let (tokens, diagnostics) = kinds("'unterminated @ ok");
        assert!(diagnostics.has_errors());
        assert_eq!(diagnostics.error_count(), 1);
        assert!(matches!(tokens[0], TokenKind::Error(_)));
        assert_eq!(tokens.last(), Some(&TokenKind::Eof));
    }

    #[test]
    fn reports_unterminated_comments() {
        let (tokens, diagnostics) = kinds("foo \"unterminated");
        assert!(diagnostics.has_errors());
        assert_eq!(
            tokens,
            vec![TokenKind::Identifier("foo".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn tracks_source_spans() {
        let mut sources = SourceManager::new();
        let source_id = sources.add_file("test.tt", "foo\n  bar");
        let mut diagnostics = DiagnosticCollector::new();
        let tokens = lex(&sources, source_id, &mut diagnostics);
        assert!(!diagnostics.has_errors());
        assert_eq!(tokens[0].span, Span::new(source_id, 0, 3));
        assert_eq!(tokens[1].span, Span::new(source_id, 6, 3));
        assert_eq!(tokens[2].span, Span::new(source_id, 9, 0));
    }
}
