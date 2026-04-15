//! tiny-talk: A Smalltalk-like language interpreter
//!
//! This crate provides a minimal interpreter for a language inspired by Smalltalk,
//! exploring message-passing and object-oriented programming concepts.

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
