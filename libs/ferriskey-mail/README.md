# FerrisKey Mail

## Overview

`ferriskey-mail` provides the email delivery infrastructure for the FerrisKey ecosystem. It abstracts the underlying transport mechanism (SMTP) to allow for reliable sending of transactional emails such as password resets, verification codes, and alerts.

## Domain & Responsibilities

This library operates within the **Notification & Communication** bounded context. Its primary responsibilities include:

- **Transport Abstraction**: Sending emails via SMTP or other configured providers.
- **Message Construction**: Building MIME messages with HTML and text bodies.
- **Configuration**: Managing SMTP settings and credentials.

## Core Components

- **Mailer**: The primary interface for sending emails.
- **EmailBuilder**: Fluent API for constructing email messages.
- **SmtpConfig**: Configuration structure for the mail transport.

## Usage

```rust
use ferriskey_mail::{Mailer, SmtpConfig, EmailBuilder};

let config = SmtpConfig::new("smtp.example.com", 587);
let mailer = Mailer::new(config)?;

let email = EmailBuilder::new()
    .to("user@example.com")
    .subject("Welcome to FerrisKey")
    .body("<h1>Hello!</h1>")
    .build()?;

mailer.send(email).await?;
```

## Dependencies

- `lettre`: The underlying Rust library for email composition and transport.