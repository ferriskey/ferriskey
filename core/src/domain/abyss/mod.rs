//! Abyss Module — external identity provider management.
//!
//! Enables federated authentication through external identity providers (OAuth2,
//! OIDC; SAML and LDAP planned). The live surface is `identity_provider` — entities,
//! ports, policies and services — plus `broker_services` for the login flow itself
//! and `federation` for provider-backed user federation.
//!
//! A second, parallel `Provider*` subsystem (`ports.rs`, `services.rs`, `policies.rs`,
//! `entities.rs`, `value_objects.rs`) used to live here: a near-literal duplicate of
//! `identity_provider`, never instantiated in the composition root and unreachable
//! from any handler. It was removed with FK-006 rather than fixed, because its policy
//! carried the same cross-realm defect as the live one and would have had to be
//! patched in lockstep forever. Its only test asserted the buggy path was correct.

pub mod broker_services;
pub mod federation;
pub mod identity_provider;
pub mod identity_provider_policies;
pub mod identity_provider_services;

pub use broker_services::BrokerServiceImpl;
pub use identity_provider_services::IdentityProviderServiceImpl;
