# FerrisKey Organization

## Overview

`ferriskey-organization` is the library responsible for Business-to-Business (B2B) organization and tenant management within the FerrisKey ecosystem.

## Domain & Responsibilities

This library operates within the **Organization & B2B Identity** bounded context. Its primary responsibilities include:

- **Organization Management**: Creating and managing organizational hierarchies and teams.
- **B2B Tenancy**: Grouping users under enterprise accounts, distinct from the global realm management.
- **Organization Policies**: Defining organization-specific security policies (e.g., forced SSO, domain matching).

## Core Components

- **Organization**: The primary entity representing a B2B customer or tenant.
- **Member**: A user linked to an organization with specific organizational roles.
- **Domain Identity**: Validating and mapping email domains to organizations for automatic onboarding.

## Technical Details

This module builds on top of `ferriskey-domain` to add an additional layer of grouping tailored for SaaS B2B applications. It handles complex relationships, such as users belonging to multiple organizations and organization-specific role assignments (RBAC scoped to the organization).

## Dependencies

- `ferriskey-domain`: Core user and role entities.
