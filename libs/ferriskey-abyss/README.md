# FerrisKey Abyss

## Overview

`ferriskey-abyss` is the **Identity Provider Federation** engine for the FerrisKey ecosystem. It bridges the gap between FerrisKey and external identity providers (IdPs), allowing users to bring their existing identities from systems like Google, GitHub, Discord, or enterprise SAML/OIDC providers.

## Domain & Responsibilities

This library operates within the **Identity Federation** bounded context. Its primary responsibilities include:

- **OpenID Connect Federation**: Acting as a relying party to authenticate users via upstream OIDC providers.
- **SAML 2.0 Bridge**: Integrating with legacy enterprise identity systems (upcoming).
- **Social Login (OAuth 2.0)**: Handling authentication flows with popular social networks.
- **Identity Mapping & Transform**: Normalizing claims from various external providers into FerrisKey's internal identity model.

## Core Components

- **IdentityProvider**: The core entity representing an upstream IdP configuration.
- **Federation Flow**: Handles the redirects, callbacks, and token exchanges required by OIDC/OAuth2 protocols.
- **Claim Mappers**: Translates external assertions into FerrisKey user attributes.

## Technical Details

The library implements a robust state machine for authentication flows (`identity_provider.rs`). It securely handles OAuth2 states and nonces to prevent CSRF and replay attacks, bridging ephemeral credentials from external providers into persistent FerrisKey identities.

## Dependencies

- `ferriskey-domain`: Core domain types and traits.
- `maskass`: Utility for masking sensitive data during logging.
