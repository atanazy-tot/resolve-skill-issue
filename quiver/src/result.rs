//! The crate-local result alias.

use crate::Error;

/// A result specialised to the crate's [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;
