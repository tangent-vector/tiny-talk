# tiny-talk

A tiny Smalltalk-like language interpreter.

## Overview

tiny-talk is a minimal interpreter for a language inspired by Smalltalk. The goal is to explore the core concepts that make Smalltalk elegant:

- **Pervasive message-passing**: Everything is an object, and computation happens by sending messages to objects.
- **"Turtles all the way down" OOP**: Objects, classes, and even the runtime itself follow the same consistent model.

This is a learning/experimental project meant to distill these ideas into a small, understandable codebase.

## Roadmap

### Phase 1: Tree-Walking Interpreter
- [ ] Minimal parser producing an AST
- [ ] Tree-walking interpreter that evaluates the AST directly
- [ ] Basic object model with message dispatch

### Phase 2: Bytecode VM (Future)
- [ ] Bytecode representation
- [ ] Stack-based virtual machine
- [ ] Compiler from AST to bytecode

## Building

```bash
cargo build
```

## Running Tests

```bash
cargo test
```

## License

MIT License - see [LICENSE](LICENSE) for details.
