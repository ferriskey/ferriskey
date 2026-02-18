# Contributing to FerrisKey

Thank you for your interest in contributing to FerrisKey! 🦀

FerrisKey is an open-source Identity and Access Management (IAM) system written in Rust and React. We welcome contributions from anyone willing to help improve it.

---

## 🧭 Where to Start

- 📖 Read the [FerrisKey Docs](https://docs.ferriskey.rs/getting-started/introduction)
- 🐞 Look at the [open issues](https://github.com/ferriskey/ferriskey/issues)
- 💬 Join discussions on [Discord](https://discord.gg/WVV5rq8ANb)
- 📝 Check our [GOVERNANCE.md](./GOVERNANCE.md)

---

## 🧪 Code Contributions

We use GitHub pull requests for code review.

*Prerequisites:*
- [Rust](https://rustup.rs/)
- [NodeJS](https://nodejs.org/en/download)
- [pre-commit](https://pre-commit.com/)
- [docker](https://docs.docker.com/engine/install/)

1. Fork the repository
2. Install pre-commit
```bash
npm install -g pnpm
pre-commit install
cd front
pnpm install
```
3. Create a branch (`feat/...`, `fix/...`, `docs/...`)
4. Implement your changes
5. Run tests with `cargo test`
6. Open a PR and fill in the description

Make sure to follow Rust best practices, and try to include unit tests when relevant.

---

## 👩‍🔬 Experimental Features

Ferris key uses feature flags to enable experimental features. For now they are only available on the frontend.

To enable a feature, add the following to your `.env` file at `/front` :

```env
VITE_FEATURES="realm-settings"
```

To look for the features in the code, please refer to the `lib/features.ts` file.

---

## 🔀 Pull Request Guidelines

### Philosophy
FerrisKey is built with a clean architecture mindset.

Each layer has a clear responsibility:
- `core` -> domain & business logic
- `api` -> transport & HTTP concerns
- `front` -> user interface
- `operator` -> deployment & infrastructure

We expect pull requests to respect this separation.

### 🧩 1. One Responsibility per Pull Request
A Pull Request must:
- Address one concern
- Modify one architectural layer whenever possible
- Be small enough to review in < 20 minutes

✅ Good Examples
- Add PKCE validation in `core`
- Add `/introspect` route in `api`
- Add TOTP setup UI in `front`
- Fix SQL constraint in `core` repository

❌ Bad Examples
- Add PKCE support + modify DB schema + update frontend + refactor token logic
- "Refactor OAuth + improve performance + fix tests"

If your change touches multiples layers:
➡️ Split it into multiple PRs


### 🏗 2. Split by Layer

Prefer:
```code
PR 1 → Core: Add domain support
PR 2 → API: Expose route using new domain feature
PR 3 → Frontend: UI integration
```


Instead of:
```code
❌ One massive PR touching core + api + frontend
```

### 📦 3. Size Matters

A PR will likely be refused if:
- It changes too many files
- It mixes refactor + feature
- It introduces unrelated formatting changes
- It includes multiple features

Rule of thumb:
> If your PR title needs “and”, it should probably be split.


### 🧠 4. Reviewability First

Before opening a PR, ask yourself:
- Can someone understand this in 10–20 minutes?
- Does it introduce hidden side effects?
- Is it scoped clearly?

If not → split.


### 🚫 5. PRs That Will Be Rejected

We reserve the right to reject PRs that:
- Break architectural boundaries
- Mix domain and transport logic
- Introduce large unscoped refactors
- Bundle multiple responsibilities

Even if the code works.


### 🪵 6. Commit Hygiene

We encourage:
- Small commits
- Clear commit messages
- Conventional commit style (recommended)

---

## 📝 Feature Proposals

For large changes, open an issue first and discuss the proposal with the maintainers. We may ask you to write an RFC if it's an architectural change.

---

## 🧹 Code Style

- Rust: `rustfmt` and `clippy`
- Frontend: TypeScript + Prettier
- Use meaningful commit messages (see [Conventional Commits](https://www.conventionalcommits.org/))

---

## 🧾 License

All contributions must be made under the [Apache 2.0 License](./LICENSE).

By submitting code, you agree to license it under this license.

---

Thanks for contributing to FerrisKey!
