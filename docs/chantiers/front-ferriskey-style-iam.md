# Chantier — migrate the IAM console to the FerrisKey style

Tracking artifact for the L feature. Written for a session with zero shared
memory: it is the single source of truth for who writes what.

- Branch: `chantier/front-ferriskey-style-iam` → `main`, one PR, squash.
- Reference: the FerrisKey style reference, rules `FK-01` … `FK-43`
  (https://claude.ai/code/artifact/bc8f0e77-30b9-4c1b-9c76-21223e83b9c2).
- Visual reference: `../ferriskey-kit/src/pages/` (neighbouring repository).
- No tracking issue — the user declined one; the PR description carries the
  decomposition.

## Shape — an alternative console under `/next/`

The migrated screens do **not** replace the current ones. They form a parallel
console mounted at `/realms/:realm_name/next/…`, living entirely under
`front/src/next/`. Nothing outside that directory is modified beyond three
additive points (see convergence below).

Consequences, all of them good:

- `/console` (CIAM) keeps its rendering **by construction** — no file it
  depends on is touched. The `ui/legacy/` strategy considered earlier is
  dropped; it is no longer needed.
- The current IAM console keeps working while the migration runs. Both are
  reachable, so a screen can be compared side by side.
- Rollback is deleting a directory and one route.

`/realms/:realm_name/next/…` rather than `/next/realms/…`: the realm stays the
first path segment, where `realmFromPath` in `App.tsx`, every
`useParams<RouterParams>()` and every existing URL helper already expect it.

The `feature/` ⇄ `ui/` split is reproduced inside `next/`: each domain gets its
own `feature/` (data, hooks, mutations, handlers) and `ui/` (presentation,
props only). The features **reuse the existing API layer** (`@/api/*.api.ts`),
schemas and validators verbatim — only the views are new.

## Frozen contracts (orchestrator-owned, read-only for workstreams)

- `front/src/index.css` — `--color-fk-*`, `--font-mono-ui`, `tnum`,
  `:root[data-style='ferriskey']`.
- `front/src/styles/style-tokens.ts` — `tokens`, `StyleTokens`.
- `front/src/components/kit/**` — the ported kit.
- `front/src/components/ui/**` — shadcn, untouched.
- `front/src/App.tsx` — the `next/*` route.
- `front/src/next/shell/**`, `front/src/next/routes.ts` — the shell and the URL
  helpers. A domain adds its own `NEXT_<DOMAIN>_URL` helpers by **reporting the
  lines**, and its sidebar entry by reporting `ready: true` on its nav item.
- `front/src/next/next-app.tsx` — the route table.
- Everything outside `front/src/next/` — the current console and the CIAM.

A workstream needing a line in any of those **reports the exact line** instead
of writing it.

## Regenerated at integration

- `front/src/api/api.client.ts` — generated from the OpenAPI document
  (`typed-openapi`). Nobody hand-edits it; it is not touched by this chantier.

## Decomposition

Every workstream owns `front/src/next/pages/<domain>/**` and nothing else.
Existing files are **read-only** for every workstream, without exception.

| # | Domain | Status |
|---|---|---|
| 0 | Foundations (tokens, kit, `/next` shell) | integrated |
| 1 | `role` — pilot, the canonical example to imitate | integrated |
| 2 | `client-scope` | integrated |
| 3 | `user` + `account` | integrated |
| 4 | `client` | integrated |
| 5 | `identity-providers` ⚠ shared with CIAM | integrated |
| 6 | `user-federation` | integrated |
| 7 | `email-template` ⚠ shared with CIAM | integrated |
| 8 | `realm` (realm-settings + webhooks) | integrated |
| 9 | `compass` + `seawatch` | integrated |
| 10 | `portal` + `portal-theme` + `portal-layouts` ⚠ themes shared | integrated |
| 11 | `overview` + `organization` | integrated |

## Verification regime

Workstreams run **concurrently in a shared checkout**. A full-tree check there
tests a tree carrying siblings' unfinished work, so it fails for reasons no
agent can act on. Therefore:

- Sub-agents run `pnpm exec tsc --noEmit -p tsconfig.app.json` and filter the
  output to their own paths, plus `pnpm exec eslint src/next/pages/<domain>`.
- `pnpm build` and `pnpm lint` in full belong to the orchestrator, after each
  integration.

Node 24 is required (`nvm use 24`); ESLint 10 crashes on Node 20.8 with
`util.styleText is not a function`.

## Server-side filtering

An audit of every `GET` endpoint accepting query parameters found **no
free-text search on any listing**. The only `search` in the API belongs to
`/organizations/{id}/groups/{gid}/members`. Structured filters exist on two
resources only:

- `/compass/v1/flows` — `client_id`, `user_id`, `grant_type`, `status`,
  `limit`, `offset`
- `/seawatch/v1/security-events` — `actor_id`, `client_id`, `event_types`,
  `from_timestamp`, `to_timestamp`, `limit`, `offset`

`ListingPage` therefore carries both modes. `server` disables the local
predicates for the keys the API handles; `useListingQuery` keeps the query and
the filter in the URL and debounces the search box at 300 ms. Compass and Sea
Watch use it; the other eleven listings filter in memory and will switch one
line at a time as the endpoints grow parameters.

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
- **An alternative console rather than an in-place rewrite.** The user asked
  for `/next/` explicitly. It removes the CIAM risk entirely and makes both
  renderings comparable while the migration runs. Rejected: replacing the views
  in place with a `legacy/` copy for the three CIAM-shared trees — more
  invasive, and it left `/console` one mistake away from changing.
- **No rationale comments in the code.** The kit annotates every deviation;
  the user asked for none here. Intent is recorded in this document and in
  `docs/chantiers/rules/<domain>.md` instead.
- **The email and portal builders keep the product implementation.** The kit
  rendered them incorrectly; only the chrome around them (page header, tabs,
  sections, panels) is restyled. Their canvas, block library and validation
  logic are untouched.
