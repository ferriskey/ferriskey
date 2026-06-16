# FerrisKey Mail

## Overview

`ferriskey-mail` is the email delivery library for the FerrisKey ecosystem. It is responsible for sending transactional emails critical to the identity lifecycle, such as email verification, magic links, and password resets.

## Domain & Responsibilities

This library operates within the **Communication & Notifications** bounded context. Its primary responsibilities include:

- **Template Management**: Formatting and rendering email templates with dynamic context.
- **SMTP Delivery**: Securely connecting to upstream mail servers and transmitting emails.
- **Transactional Emails**: Orchestrating identity-related emails (welcome, verification, recovery).

## Core Components

- **MailService**: The primary interface for dispatching emails.
- **Templates**: Structured payloads and raw HTML/Text content formatting.

## Technical Details

The library provides an asynchronous, non-blocking interface for email dispatch, ensuring that authentication flows or background jobs are not bottlenecked by SMTP communication latency. It handles TLS/SSL configurations and authentication with upstream mail providers.

## Dependencies

- `ferriskey-domain`: To extract user email addresses and realm configurations.
- `lettre` (or similar): For SMTP transport and email builder utilities.
