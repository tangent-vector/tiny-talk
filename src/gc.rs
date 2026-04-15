//! # Garbage Collection
//!
//! Memory management for the VM's object space.
//!
//! ## Responsibilities
//!
//! - **Reachability analysis**: Determine which objects are reachable from the
//!   root set (global variables, stack frames, other known-live references).
//!
//! - **Reclamation**: Free memory used by unreachable objects so it can be
//!   reused for new allocations.
//!
//! - **Handle validity**: Manage the relationship between handles and object
//!   storage, potentially invalidating or updating handles after collection.
//!
//! - **Collection triggering**: Decide when to run garbage collection (memory
//!   pressure, allocation count, explicit request).
//!
//! ## Dependencies and Relationships
//!
//! This module depends on:
//! - `vm`: accesses object storage and needs to understand object layout
//!
//! This module is used by:
//! - `vm`: triggers collection and provides the root set
//! - `eval`: may interact with GC during evaluation (safepoints, write barriers)
//!
//! ## Architectural Approach
//!
//! ### Current Status: Placeholder
//!
//! For the initial tree-walking interpreter, garbage collection is **not yet
//! implemented**. Objects are allocated but never freed. This is acceptable for:
//!
//! - Small test programs
//! - Learning the language semantics
//! - Getting the rest of the system working
//!
//! Eventually, GC will be necessary for non-trivial programs.
//!
//! ### Future: Mark-and-Sweep
//!
//! The simplest collector suitable for tiny-talk is **mark-and-sweep**:
//!
//! 1. **Mark phase**: Starting from roots, traverse all reachable objects and
//!    set a "marked" flag on each.
//!
//! 2. **Sweep phase**: Scan all objects; those not marked are unreachable and
//!    can be freed. Clear marked flags for the next collection.
//!
//! This stop-the-world approach is simple to implement correctly.
//!
//! ### Root Set
//!
//! The root set includes:
//! - Global variables (class references, global bindings)
//! - The evaluation stack (local variables, temporaries, arguments)
//! - In-progress message sends (receiver, arguments)
//!
//! The VM must expose this set to the GC module.
//!
//! ### Handle Tables
//!
//! With handle-based references, GC interacts with the handle table:
//!
//! - Handles remain valid after GC (they're indices, not pointers)
//! - Dead objects have their handle slots marked as free
//! - A **free list** tracks reusable slots
//!
//! Alternatively, a **compacting** collector could move objects and update
//! handles, but this adds complexity.
//!
//! ### Write Barriers (Future)
//!
//! More sophisticated collectors (generational, concurrent) need **write barriers**—
//! code that executes when an object reference is modified. The VM's object
//! storage operations would call into the GC module.
//!
//! This is not needed for basic mark-and-sweep.
//!
//! ### Integration Points
//!
//! The GC module will eventually provide:
//!
//! - A `collect()` function triggered by the VM
//! - Hooks for the VM to register roots
//! - Possibly a `Gc<T>` wrapper type for safe handle management
//!
//! For now, this module serves as a placeholder documenting the planned approach.

// TODO: Implement garbage collection (not urgent for initial interpreter)
