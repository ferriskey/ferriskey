# Chantier — migrate the IAM console to the FerrisKey style

Tracking artifact for the L feature. Written for a session with zero shared
memory: it is the single source of truth for who writes what.

- Branch: `chantier/front-ferriskey-style-iam` → `main`, one PR, squash.
- Reference: the FerrisKey style reference, rules `FK-01` … `FK-43`
  (https://claude.ai/code/artifact/bc8f0e77-30b9-4c1b-9c76-21223e83b9c2).
- Visual reference: `../ferriskey-kit/src/pages/` (neighbouring repository).
- No tracking issue — the user declined one; the PR description carries the
  decomposition.

## Scope boundary — IAM yes, CIAM no

`front/src/App.tsx` mounts two products. Everything under `<Layout />` is in
scope; everything under `<Route path='console' element={<ProductLayout />}>`
is out, and must keep its current rendering byte for byte.

Three IAM trees are mounted verbatim on the CIAM side:

| CIAM page | Reuses |
|---|---|
| `console-branding` | `email-template/feature/*`, `portal/themes/feature/*` |
| `console-authentication` | the whole `PageIdentityProviders` |

Strategy, decided before dispatch: the current `ui/page-x.tsx` is **moved** to
`ui/legacy/page-x.tsx` untouched, the new view takes its place, and the
`*-feature.tsx` carries one line — `pathname.includes('/console/') ? <Legacy/>
: <New/>`. Phase 2 (CIAM migration) deletes `legacy/` and that line.

## Frozen contracts (orchestrator-owned, read-only for workstreams)

- `front/src/index.css` — `--color-fk-*`, `--font-mono-ui`, `tnum`,
  `:root[data-style='ferriskey']`.
- `front/src/styles/style-tokens.ts` — `tokens`, `StyleTokens`.
- `front/src/components/kit/**` — the ported kit.
- `front/src/components/ui/**` — shadcn, untouched.
- `front/src/App.tsx`, `components/layout/**`, `app-sidebar.tsx`, `nav-*.tsx`,
  every `pages/console-*`.

A workstream needing a line in any of those **reports the exact line** instead
of writing it.

## Regenerated at integration

- `front/src/api/api.client.ts` — generated from the OpenAPI document
  (`typed-openapi`). Nobody hand-edits it; it is not touched by this chantier.

## Decomposition

Every workstream owns `front/src/pages/<domain>/ui/**`, its
`front/src/routes/sub-router/<domain>.router.ts`, and its `*-feature.tsx`
files — the latter for **additive** changes only (a missing datum, the legacy
switch), never a reorganisation.

| # | Domain | Status |
|---|---|---|
| 0 | Foundations (tokens, kit, `data-style` flag) | integrated |
| 1 | `role` | pending |
| 2 | `client-scope` | pending |
| 3 | `user` + `account` | pending |
| 4 | `client` | pending |
| 5 | `identity-providers` ⚠ shared with CIAM | pending |
| 6 | `user-federation` | pending |
| 7 | `email-template` ⚠ shared with CIAM | pending |
| 8 | `realm` (realm-settings + webhooks) | pending |
| 9 | `compass` + `seawatch` | pending |
| 10 | `portal` + `portal-theme` + `portal-layouts` ⚠ themes shared | pending |
| 11 | `overview` + `organization` | pending |

## Verification regime

Workstreams run **concurrently in a shared checkout**. A full-tree check there
tests a tree carrying siblings' unfinished work, so it fails for reasons no
agent can act on. Therefore:

- Sub-agents run `pnpm exec tsc --noEmit -p tsconfig.app.json` and filter the
  output to their own paths, plus `pnpm exec eslint src/pages/<domain>`.
- `pnpm build` and `pnpm lint` in full belong to the orchestrator, after each
  integration.

Node 24 is required (`nvm use 24`); ESLint 10 crashes on Node 20.8 with
`util.styleText is not a function`.

## Decision log

- **One style object, not a provider.** The kit shipped four styles and a
  `StyleProvider`; one was acted. A context that can only ever return one
  value is indirection without abstraction. Rejected: keeping the provider
  "in case a second style comes back" — the rule of three says no.
- **Style scoped by `data-style` on `<html>`, not global CSS.** Overriding
  `--primary` at `:root` would repaint every CIAM button too. Rejected:
  scoping to the page container — portals render outside the shell and would
  have kept the old radius.
- **Four kit components not ported** (`SaveBar`, `DangerZone`, `ProviderLogo`,
  `DurationInput`): the product already has richer equivalents. Rejected:
  porting them for kit fidelity — that is a duplicate, not a convergence.
- **The email and portal builders keep the product implementation.** The kit
  rendered them incorrectly; only the chrome around them (page header, tabs,
  sections, panels) is restyled. Their canvas, block library and validation
  logic are untouched.
