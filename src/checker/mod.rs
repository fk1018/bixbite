/// Built-in no-op type checker backend.
pub mod noop;
/// Type checker trait definition.
#[path = "trait.rs"]
pub mod type_checker_trait;

/// Re-export of the type checker trait.
pub use type_checker_trait::TypeChecker;
