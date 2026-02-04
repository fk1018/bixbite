/// Emitter trait definition.
#[path = "trait.rs"]
pub mod emitter_trait;
/// Ruby emitter implementation.
pub mod ruby;

/// Re-export of the emitter trait.
pub use emitter_trait::Emitter;
