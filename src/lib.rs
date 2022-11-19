//! [FinPartOrd] is a trait for representing finite partial orders.

#[cfg(feature = "dag")]
mod dag;
#[cfg(feature = "dag")]
pub use dag::*;

#[cfg(feature = "pairs")]
mod pairs;
#[cfg(feature = "pairs")]
pub use pairs::*;

mod r#trait;
pub use r#trait::*;
