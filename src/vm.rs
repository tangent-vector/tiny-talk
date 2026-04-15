//! # Virtual Machine
//!
//! Represents runtime entities: objects, classes, methods, and their relationships.
//!
//! ## Responsibilities
//!
//! - **Object representation**: Define how objects are stored in memory, including
//!   their class reference and instance variables.
//!
//! - **Class representation**: Define classes with their method dictionaries,
//!   instance variable layout, and superclass chain.
//!
//! - **Method storage**: Store method implementations (AST-based for the tree-walking
//!   interpreter) in a way that supports message dispatch.
//!
//! - **Message dispatch**: Look up methods by selector, walking the superclass chain
//!   as needed (the core of Smalltalk's computation model).
//!
//! - **Primitive operations**: Provide built-in operations for fundamental types
//!   (arithmetic, comparison, I/O) that can't be defined in tiny-talk itself.
//!
//! - **Object identity and equality**: Support both identity comparison (`==`) and
//!   value equality (`=`).
//!
//! ## Dependencies and Relationships
//!
//! This module depends on:
//! - `ast`: for method body representation (in a tree-walking interpreter)
//! - `source`: for locations attached to methods (for stack traces)
//!
//! This module is used by:
//! - `eval`: uses the VM's object model to execute programs
//! - `gc`: will traverse and manage VM objects
//!
//! ## Architectural Approach
//!
//! ### Centralized Object Storage
//!
//! All objects are owned by a central **VM** (or **Heap** or **ObjectSpace**) structure.
//! This enables:
//!
//! - Safe garbage collection (one place to scan)
//! - Object identity based on stable handles/IDs
//! - No Rust lifetime complexity from inter-object references
//!
//! ### Handle-Based References
//!
//! Objects reference each other through **handles** (or **ObjRef**, **ObjectId**)—small
//! copyable identifiers that the VM can resolve to actual object data. This is similar
//! to how a database uses row IDs rather than memory pointers.
//!
//! Benefits:
//! - Objects can be moved by GC without invalidating references
//! - Handles can be compared for identity cheaply
//! - The system naturally detects invalid/stale references
//!
//! ### Object Layout
//!
//! A typical object contains:
//! - A reference to its class
//! - A vector of instance variables (also handles)
//! - Optionally, a payload for special objects (strings, numbers, arrays)
//!
//! ### Class Structure
//!
//! A class object contains:
//! - A reference to its superclass (or a sentinel for the root)
//! - A method dictionary (selector → method)
//! - Instance variable names (for reflection and debugging)
//! - Class-side methods (for class methods vs instance methods)
//!
//! ### Method Dispatch
//!
//! Message dispatch is the heart of Smalltalk execution:
//!
//! 1. Extract the receiver's class
//! 2. Look up the selector in the class's method dictionary
//! 3. If not found, walk up the superclass chain
//! 4. If still not found, send `doesNotUnderstand:` to the receiver
//! 5. If found, invoke the method with the receiver and arguments
//!
//! ### Primitive Methods
//!
//! Some methods need to be implemented in Rust, not tiny-talk:
//!
//! - Arithmetic on small integers
//! - String manipulation
//! - I/O operations
//! - Object instantiation
//!
//! These are **primitives**, identified by a primitive index. When a primitive
//! fails, it can fall back to the method's tiny-talk body.
//!
//! ### Immediate Values (Optimization)
//!
//! Small integers and other common values might be represented as **immediate**
//! values—encoded directly in the handle rather than requiring a heap allocation.
//! This is a common optimization but can be deferred to later versions.
//!
//! ### Built-in Objects
//!
//! The VM pre-creates essential objects:
//! - `nil`: the undefined/nothing value
//! - `true` and `false`: boolean values
//! - Core classes: `Object`, `Class`, `SmallInteger`, `String`, `Array`, `Block`
//!
//! These are available immediately when evaluation begins.

// TODO: Implement VM object model
