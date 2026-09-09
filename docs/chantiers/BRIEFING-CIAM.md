# Briefing — rebuilding one CIAM section in the `/next` console

You are one of four agents, each rebuilding one section of the customer
console. You share a checkout. Read this file entirely before touching
anything, **then read `docs/chantiers/BRIEFING.md`** — the constraints of the
IAM migration apply here unchanged, and this file only states what differs.

## What you are doing

`front/` is the React console of FerrisKey. It ships two products: an admin
console (IAM) and a customer console (CIAM, mounted under `/console`). Both are
being rebuilt in the "FerrisKey" style under `front/src/next/`. The IAM is
done. Yours is the CIAM.

The new console lives at `/realms/:realm_name/next/console/…`. Its shell —
top bar, section rail, sub-item rail — is built and frozen. Your route and your
sidebar entries already exist and point at a placeholder; you replace it.

## Read these first

1. `docs/chantiers/BRIEFING.md` — the shared rules. All of them still apply:
   the `feature/` ⇄ `ui/` split, the business-rule inventory, no comments in
   code, English strings, verbatim reuse of the API layer, divergences
   reported rather than fixed, and the verification commands.
2. `front/src/next/pages/iam/role/` — still the canonical example of layering.
3. `front/src/components/kit/index.ts` — the frozen component contract.
4. `front/src/next/shell/ciam/nav-config.ts` — the section map you belong to.
5. The screens you are rebuilding, named in your mission, under
   `front/src/pages/`.

## What differs from the IAM chantier

- **There is no prototype for the CIAM.** `ferriskey-kit` only covered the
  admin console. Your reference for *structure* is the current CIAM screen, and
  your reference for *appearance* is the FerrisKey style as it now stands in
  `components/kit/` — read a finished IAM screen to see what that means in
  practice. When the current screen and the style disagree, the style wins on
  appearance and the current screen wins on behaviour.
- **Denser than the IAM.** The console shell already is; keep the content in
  step. 13 px is the body size of a table or a rail, `text-sm` is for form
  fields, and density comes from padding rather than from shrinking text.
- **Sharing with the IAM is allowed**, and encouraged where the screens are
  genuinely the same — but only within `front/src/next/`. A CIAM page may
  import an IAM page, a component or a hook from `next/`; it may not reach into
  `front/src/pages/` for anything but reading, and it may not add a dependency
  on the current console.

## Scope

**You own `front/src/next/pages/ciam/<your-section>/` and
`docs/chantiers/rules/console-<your-section>.md`. Nothing else.**

Read anything; write only those. These are read-only, and a line you need in
one of them is **reported, not written**:

- `front/src/next/shell/**` — both shells, the nav config, the top bar
- `front/src/next/console-app.tsx`, `next-app.tsx`, `routes.ts`
- `front/src/next/pages/iam/**` — the finished admin console
- `front/src/components/**`, `front/src/styles/**`
- everything under `front/src/pages/` — the current console and the current CIAM

Do not touch git: no commit, no branch. The orchestrator integrates.

## Verification

```bash
cd /Users/baptiste/Development/ferrislabs/ferriskey/front
export NVM_DIR="$HOME/.nvm"; . "$NVM_DIR/nvm.sh"; nvm use 24
pnpm exec tsc --noEmit -p tsconfig.app.json
pnpm exec eslint src/next/pages/ciam/<your-section>
```

Four agents write into this checkout at once: filter the `tsc` output to your
own paths, and never run `pnpm build` or `pnpm lint` in full.

## Report format

The one in `docs/chantiers/BRIEFING.md`, unchanged.
