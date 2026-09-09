# Briefing — migrating one IAM domain to the `/next` console

You are one of ten agents, each migrating one domain. You share a checkout.
Read this file entirely before touching anything.

## What you are doing

`front/` is the React console of FerrisKey. A parallel IAM console is being
built at `/realms/:realm_name/next/…`, living entirely under
`front/src/next/`. It uses the "FerrisKey" design system, prototyped in the
neighbouring repository `../ferriskey-kit` and specified as 43 numbered rules
`FK-01` … `FK-43`.

Your job: fill `front/src/next/pages/<your-domain>/` so that your domain's
screens exist in that console, look like the corresponding screens of the
prototype, and keep every business rule the current screens carry.

Your route and your sidebar entry are already wired. A placeholder currently
sits at `front/src/next/pages/<your-domain>/page-<your-domain>.tsx`; you
replace it.

## Read these first, in this order

1. `docs/chantiers/front-ferriskey-style-iam.md` — scope, frozen contracts,
   verification regime, decision log.
2. **`front/src/next/pages/role/`** — the pilot, already merged. It is the
   canonical example: same layering, same import style, same way of splitting
   `feature/` and `ui/`, same way of reusing the existing API layer. Imitate
   it rather than inventing your own convention.
3. `front/src/components/kit/index.ts` — the frozen component contract. It
   lists what is exported **and** what was deliberately not ported, with the
   product equivalent to use instead. Then read the components you will mount.
4. `front/src/styles/style-tokens.ts` — the `tokens` object. Import it; never
   hard-code the values it holds.
5. The 43 rules: https://claude.ai/code/artifact/bc8f0e77-30b9-4c1b-9c76-21223e83b9c2
   (`WebFetch` works on this URL — it is an artifact, not a private page).
6. Your prototype screens in `../ferriskey-kit/src/pages/`, named in your
   mission.

## Two non-negotiable constraints

### 1. The `feature/` ⇄ `ui/` split

- `feature/page-x-feature.tsx` — data, hooks, mutations, handlers, URL state.
- `ui/page-x.tsx` — presentation. Receives props, holds no query, no mutation.

Features **reuse the existing API layer verbatim**: `@/api/<domain>.api.ts`,
the zod schemas in `@/pages/<domain>/schemas/`, the validators, the types from
`@/api/api.client`. You are writing views, not a new data layer.

### 2. Every business rule of the current screens survives

The current views carry logic written nowhere else: validations, conditional
fields, disabled states, context filters, error messages, imposed orders.

Procedure, per screen, in this order:

1. Read the current `front/src/pages/<domain>/ui/page-x.tsx` **in full**.
2. Write its business rules into `docs/chantiers/rules/<your-domain>.md`, as a
   numbered table — validations, `disabled`, conditional rendering,
   exclusions, help texts, imposed orders. Follow the format of
   `docs/chantiers/rules/role.md`.
3. Write the new view.
4. Re-read your list and tick every rule. **A rule not carried over is a
   migration bug, not a simplification.** If you deliberately drop one, say so
   in the table with the reason.

## How faithful to the prototype

Reproduce its **structure**: same kit components, same layout, same columns,
same metrics, same filters, same alerts, same tab order, same section order.

Three things you do not copy:

- **Its data model is fictional.** Where the prototype invents a field, use
  the real one from `@/api/api.client` and, when in doubt, from the Rust
  domain in `core/src/domain/` and `libs/ferriskey-domain/`.
- **Its strings are French.** The product console is English. Every label,
  placeholder, description, empty state and error message you write is in
  English.
- **Its corrections are not automatically right.** The prototype fixed some
  UI/domain divergences and introduced others. Check the real domain, and
  **report what you find — do not fix it, and do not reproduce the
  prototype's version either.**

## Code style

- **No comments.** None. Not a rationale block, not a `FK-nn` citation, not an
  explanation of a deviation. Everything you would have written as a comment
  goes into `docs/chantiers/rules/<your-domain>.md` and into your report.
- Match the surrounding style: single quotes, no semicolons, `cn()` for class
  composition, `import X = Schemas.X` for domain types.
- `noUnusedLocals` and `noUnusedParameters` are on: a dead import breaks the
  build.
- No `setState` inside a `useEffect` to mirror fetched data — the lint rule
  `react-hooks/set-state-in-effect` rejects it. See how
  `next/pages/role/feature/page-role-detail-feature.tsx` derives its draft at
  render time instead.

## Scope

**You own `front/src/next/pages/<your-domain>/` and
`docs/chantiers/rules/<your-domain>.md`. Nothing else.**

Read anything; write only those. In particular these are read-only:

- everything under `front/src/pages/` (the current console) and every
  `pages/console-*` (the CIAM product — it must not change);
- `front/src/next/shell/`, `front/src/next/routes.ts`,
  `front/src/next/next-app.tsx`;
- `front/src/components/kit/`, `front/src/components/ui/`,
  `front/src/styles/`, `front/src/App.tsx`;
- anything belonging to another domain under `front/src/next/pages/`.

Your URL helpers already exist in `front/src/next/routes.ts`. If you need
something added to any read-only file — a helper, a shadcn primitive, a kit
export — **stop, and report the exact line you need**. Do not write it, and do
not widen your scope.

Do not touch git: no commit, no branch, no stash. The orchestrator integrates.

## Verification

Node 24 is required; ESLint 10 crashes on the system Node.

```bash
cd /Users/baptiste/Development/ferrislabs/ferriskey/front
export NVM_DIR="$HOME/.nvm"; . "$NVM_DIR/nvm.sh"; nvm use 24
pnpm exec tsc --noEmit -p tsconfig.app.json
pnpm exec eslint src/next/pages/<your-domain>
```

Ten agents write into this checkout at the same time, so `tsc` reports the
whole tree: **filter its output to your own paths and ignore the rest**. Never
run `pnpm build` or `pnpm lint` in full — that is the orchestrator's job.

## Report format

Prose is not a report. Return exactly these sections:

```
### Files written
(exact paths, repo-relative)

### Business rules
docs/chantiers/rules/<domain>.md — N rules inventoried, M carried over.
Every rule NOT carried over, with its reason.

### Verification
The real output of the two commands, filtered to your paths.

### Deviations from the prototype
One line each: screen, what differs, why.

### UI ↔ domain divergences found
What the UI shows vs what the domain says, with the domain file as evidence.
A finding, not a fix.

### Lines needed in read-only files
Exact file, exact line. Empty if none.

### Open questions
```
