//! Abyss Module - External Identity Provider Management
//!
//! This module provides functionality for managing external identity providers (IdPs)
//! such as Google, GitHub, Discord, and enterprise providers like SAML and LDAP.
//!
//! # Overview
//!
//! The Abyss module enables federated authentication by allowing users to authenticate
//! through external identity providers. It supports:
//!
//! - **OAuth2 providers**: Google, GitHub, Discord, etc.
//! - **OpenID Connect (OIDC)**: Standard OIDC-compliant providers
//! - **SAML** (planned): Enterprise SAML identity providers
//! - **LDAP** (planned): Directory services integration
//!
//! # Architecture
//!
//! The module follows FerrisKey's hexagonal architecture:
//!
//! - `entities`: Domain entities (`Provider`, `ProviderMapping`, `ProviderType`)
//! - `ports`: Trait definitions for service, policy, and repository
//! - `services`: Business logic implementation
//! - `policies`: Authorization policy implementation
//! - `value_objects`: Input DTOs for service operations
//!
//! # Usage
//!
//! ```rust,ignore
//! use ferriskey_core::domain::abyss::{
//!     entities::{Provider, ProviderType},
//!     ports::ProviderService,
//!     value_objects::CreateProviderInput,
//! };
//!
//! // Create a new OAuth2 provider
//! let input = CreateProviderInput {
//!     realm_id,
//!     name: "Google".to_string(),
//!     provider_type: ProviderType::OAuth2,
//!     client_id: "your-client-id".to_string(),
//!     client_secret: "your-client-secret".to_string(),
//!     authorization_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
//!     token_url: "https://oauth2.googleapis.com/token".to_string(),
//!     userinfo_url: Some("https://openidconnect.googleapis.com/v1/userinfo".to_string()),
//!     scopes: vec!["openid".to_string(), "email".to_string()],
//!     configuration: serde_json::json!({}),
//! };
//!
//! let provider = provider_service.create_provider(identity, input).await?;
//! ```
//!
//! # Security Considerations
//!
//! - Client secrets are stored encrypted in the database
//! - Provider management requires `ManageRealm` permission
//! - Providers are scoped to realms for multi-tenant isolation
//! - Only enabled providers are available for authentication

pub mod entities;
pub mod federation;
pub mod policies;
pub mod ports;
pub mod services;
pub mod value_objects;
