# Roadmap

The public roadmap lives at <https://ferriskey.rs/roadmap>. It has three columns: what FerrisKey provides today, what is being built next, and longer-term explorations.

This file is the entry point from the repository. The current release is [v0.9.0](https://github.com/ferriskey/ferriskey/releases/tag/v0.9.0).

## Next steps

Work in progress is tracked in the [v0.10 milestone](https://github.com/ferriskey/ferriskey/milestone/9) and in epics:

- [Server-side pagination, sorting and filtering on every listing](https://github.com/ferriskey/ferriskey/issues/1531) (v0.10)
- [GDPR compliance and user self-service](https://github.com/ferriskey/ferriskey/issues/992) (v0.10)
- [Smart account chooser and session-aware login](https://github.com/ferriskey/ferriskey/issues/671) (v0.10)
- [Pooled realms: host many tenants as realms on one instance](https://github.com/ferriskey/ferriskey/issues/1610)
- [OAuth 2.0 Device Authorization Grant (RFC 8628)](https://github.com/ferriskey/ferriskey/issues/1020)

Every roadmap item that has started has an issue labelled [`epic`](https://github.com/ferriskey/ferriskey/issues?q=is%3Aissue+label%3Aepic).

## How the roadmap is maintained

- The maintainers review the roadmap at each minor release and move shipped items to "Current capabilities".
- Items are added or reprioritized through [GitHub Discussions](https://github.com/ferriskey/ferriskey/discussions), following the RFC process described in [GOVERNANCE.md](./GOVERNANCE.md).
- Changes to the roadmap page are made in the [ferriskey/website](https://github.com/ferriskey/website/blob/main/apps/website/src/lib/roadmap.ts) repository.
