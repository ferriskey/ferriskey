//! Domain model for OAuth 2.0 Token Exchange (RFC 8693).

pub mod entities;
pub mod ports;
pub mod services;
pub mod value_objects;

pub use entities::{TokenExchangeError, TokenType};
pub use ports::{SubjectTokenIssuer, TokenExchangeService};
pub use services::TokenExchangeServiceImpl;
pub use value_objects::{TokenExchangeInput, TokenExchangeOutput, TokenExchangeParams};
