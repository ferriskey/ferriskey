//! Domain model for OAuth 2.0 Token Exchange (RFC 8693).

pub mod entities;
pub mod value_objects;

pub use entities::{TokenExchangeError, TokenType};
pub use value_objects::{TokenExchangeInput, TokenExchangeOutput};
