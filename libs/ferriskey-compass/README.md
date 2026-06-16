# FerrisKey Compass

## Overview

`ferriskey-compass` is the **Authentication Flow Engine** for the FerrisKey ecosystem. It orchestrates multi-step authentication processes with conditional logic, managing the complex state machine required for modern, secure login flows.

## Domain & Responsibilities

This library operates within the **Authentication Orchestration** bounded context. Its primary responsibilities include:

- **Flow Management**: Driving the user through multi-step authentication (e.g., password -> TOTP -> consent).
- **Conditional Logic**: Dynamically determining the next required authentication step based on user context, risk signals, and realm policies.
- **Event Recording**: Emitting detailed flow metrics and step recordings for auditing and anomaly detection.

## Core Components

- **CompassFlow**: The entity representing an ongoing authentication attempt, tracking its state, user agent, and IP.
- **FlowRecorder**: A thread-safe, asynchronous recorder (`recorder.rs`) that publishes flow events (started, step recorded, completed) via channels.
- **StepStatus / FlowStatus**: Enums defining the outcome of individual authentication challenges.

## Technical Details

The library uses a highly asynchronous, event-driven architecture using `tokio::sync::mpsc` channels to decouple flow execution from audit logging. This ensures that recording authentication steps (`CompassFlowStep`) does not block the critical path of the login process.

## Dependencies

- `ferriskey-domain`: Defines the core entities like `RealmId` and `Uuid`.
- `tokio`: Used for asynchronous channels (`mpsc`, `oneshot`) to handle event recording.
- `chrono`: For precise timestamping of authentication steps.
