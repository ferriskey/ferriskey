# FerrisKey Aegis

## Overview

`ferriskey-aegis` is the **Scopes & Protocol Mappers** engine of the FerrisKey ecosystem. It acts as the authorization boundary that controls exactly what data flows into your access tokens and ID tokens.

## Domain & Responsibilities

This library operates within the **Authorization & Token Minting** bounded context. Its primary responsibilities include:

- **Protocol Mappers**: Transforming internal user attributes and roles into standard OIDC/SAML claims.
- **Scope Management**: Evaluating requested scopes against user consents and client allowed scopes.
- **Custom Claims**: Providing extensibility to inject custom business logic into token generation.

## Core Components

- **Mapper**: The core trait/structure for executing transformations from domain entities to token claims (`ports.rs` / `entities.rs`).
- **ScopeEvaluator**: Logic to resolve overlapping scopes and determine final token permissions.
- **Claim Extractor**: Mechanisms to safely pull data from the user profile, roles, or realm configuration.

## Technical Details

The library heavily leverages FerrisKey's internal data model to securely build tokens. It evaluates `AccessPolicy` and scopes before issuing credentials, ensuring that a client application only receives the claims it has been explicitly granted.

## Dependencies

- `ferriskey-domain`: Provides the user, role, and resource definitions required for scope and claim evaluation.
