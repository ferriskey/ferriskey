# Business rules inventory — CIAM `user-management`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Source screens: `front/src/pages/user-management/` — 9 files under `ui/`,
9 under `feature/`, 1 router. 3 856 lines.

## Sharing decisions

Sharing is allowed inside `next/`. One call per sub-item:

| Sub-item | Same product need as the admin console? | What is mounted |
|---|---|---|
| Identities | **No** for the listing, **yes** for the detail and the creation | CIAM listing written here (`ui/page-identities.tsx`); detail and creation mount `next/pages/iam/user/ui/*` |
| Organizations | **Yes** | mounts `next/pages/iam/organization/ui/*` |
| Roles | **Yes** | mounts `next/pages/iam/role/ui/*` |

**Why the identities listing differs, in one line:** the admin listing shows a
population of realm accounts including service accounts, keyed on account type;
the CIAM listing shows the customer population only — service accounts filtered
out — and replaces the `Type` column with `Signed up` and `Roles`, and the
account-type filter with a *pending actions* dimension the admin screen does not
surface at all.

Everything else — organizations, roles, the identity detail, the identity
creation — has the same columns, the same actions and the same detail on both
sides, so the admin screen is mounted rather than cloned. Only the `feature/`
layer is written here, because the console owns a different URL space
(`/next/console/user-management/…` instead of `/next/users`, `/next/roles`,
`/next/organizations`) and the IAM features hard-code the IAM URLs.

## `ui/page-identities.tsx` → rewritten as `ui/page-identities.tsx`

| # | Rule | Carried over |
|---|---|---|
| I1 | Service accounts are excluded from the list (CIAM = customer identities) | ✅ `isServiceAccount` filter in the feature |
| I2 | Display name = `firstname lastname`, falling back to `username` | ✅ |
| I3 | Username is shown under the name, as a secondary line | ✅ mono-ui, FK-35 |
| I4 | Status precedence: disabled wins over pending actions, which wins over active | ✅ same order in the `status` column |
| I5 | Pending-action count is pluralised (`1 action` / `2 actions`) | ✅ |
| I6 | Email column shows a verified / unverified marker, only when an email exists | ✅ `MailCheck` / `MailX`; `no email` otherwise |
| I7 | Empty email renders `—` | ⚠️ changed to the literal `no email` — FK-32, the absence is named, and it matches the admin listing |
| I8 | Created column is a relative date (`just now`, `5m ago`, `3d ago`, then a date) | ✅ `formatRelative` from `@/next/shared/format-date`; no new formatter |
| I9 | Roles column is the count of `roles`, right-aligned, tabular | ✅ `align: 'right'`, `tnum` |
| I10 | Four statistics: total, email verified, pending actions, disabled | ✅ `metrics` |
| I11 | Status filters: All, Active, Unverified, Pending actions, Disabled | ✅ `filters`; `All` is `ListingPage`'s built-in default |
| I12 | Search covers username, email, firstname, lastname | ✅ `searchIn` |
| I13 | Sorting by most recent / name / email | ⚠️ replaced by column sorting (`sortValue` on identity, email, status, signed up, roles) — the kit listing sorts by header, there is no separate sort select. Strictly more, not less |
| I14 | Footer line `Showing X of Y` and `N verified · N pending action(s)` | ✅ `aggregates` on the identity and status columns |
| I15 | Empty state distinguishes "no identity at all" from "no match" | ✅ `ListingPage` shows `emptyLabel` / `emptyHint` for the empty set and its own no-match state for a filtered one |
| I16 | Loading renders skeleton rows, not an empty state | ✅ `loading` |
| I17 | Clicking a row opens the identity | ✅ `getHref` → `…/identities/:user_id/overview` |
| I18 | Primary action `Create identity` | ✅ relabelled `New identity`, to match the console's other listings |
| — | New: an alert names the identities blocked at next sign-in and links to the first one | added — the pending-actions dimension was a metric only, never actionable |

## `ui/page-identity-detail.tsx` → mounts `iam/user/ui/page-user-detail.tsx`

| # | Rule | Carried over |
|---|---|---|
| D1 | Profile fields: firstname, lastname, email — all optional | ✅ overview tab |
| D2 | Email validated against a mail-shaped regex, error `Enter a valid email address.` | ✅ through `updateUserValidator`, the API-layer validator — the message is the validator's |
| D3 | `Email verified` and `Account enabled` toggles, saved with the profile | ✅ `SwitchField` |
| D4 | Save is disabled unless the form is dirty **and** the email is valid | ✅ the SaveBar only shows when `dirtyCount > 0`, and `save()` returns early when the parse fails |
| D5 | Reset restores every field to the loaded identity | ✅ `Discard` |
| D6 | Required actions are toggled from a catalogue of four (`configure_passkey`, `configure_otp`, `update_password`, `verify_email`) and saved with the profile | ✅ the overview tab carries the same catalogue |
| D7 | A banner counts the actions currently persisted | ⚠️ dropped — the header pills and the overview tab already state it; the banner restated the same number twice on one screen |
| D8 | Enrolled credentials are listed read-only, typed (passkey, TOTP, magic link, recovery codes, federated, password) with their creation date | ✅ Credentials tab |
| D9 | A password or secret is never displayed, only replaced | ✅ FK-34 — the credentials tab offers a reset form and no read |
| D10 | Roles are assigned and unassigned from the detail, with a searchable candidate list excluding the already-assigned | ✅ Role mapping tab, through `EntityPicker` (FK-28) instead of a bespoke free-text list |
| D11 | Account info shows identity id, username, created, last update | ✅ header (`id` in grey mono, `Created`) + overview tab |
| D12 | Dates use a medium date + short time format | ✅ `formatDate` from `@/next/shared/format-date` |
| D13 | Delete asks for an in-place confirmation before firing | ✅ `DangerZone` + `ConfirmDestructiveDialog` |
| D14 | Delete redirects to the identity list | ✅ redirects to the **console** list |
| D15 | Loading renders a skeleton header, not an empty page | ✅ |
| — | New: organizations and attributes tabs | gained from the admin screen; the CIAM detail had neither |
| — | Tab state is a URL segment | ✅ FK-16, `useRouteTabs` on the console base path |

## `ui/page-create-identity.tsx` → mounts `iam/user/ui/page-create-user.tsx`

| # | Rule | Carried over |
|---|---|---|
| C1 | Username is required; every other field is optional | ✅ `createUserValidator` |
| C2 | Username is auto-derived from the email local part, else from `firstname.lastname`, slugified | ⚠️ dropped — the admin creation screen asks for the username explicitly and states it cannot be changed afterwards. Silently deriving a permanent identifier from a mutable field is the kind of surprise the console avoids. Reported, not reinstated |
| C3 | A manual edit of the username stops the derivation | ⚠️ dropped with C2 |
| C4 | Email validated with the same regex, blocking submission | ✅ validator |
| C5 | `Mark email as verified` checkbox, off by default | ✅ `SwitchField`, off by default |
| C6 | Submit disabled while submitting, label becomes `Creating…` | ✅ the SaveBar only shows once the form is valid and dirty |
| C7 | Cancel returns to the identity list | ✅ to the **console** list |
| C8 | On success: toast then redirect to the list | ✅ |

## `ui/page-organizations.tsx` → mounts `iam/organization/ui/page-organizations-overview.tsx`

| # | Rule | Carried over |
|---|---|---|
| O1 | Cards show name, `@alias`, description, domain, created | ✅ list and card views; `Created` moves to the detail header |
| O2 | Disabled organizations carry an `Off` badge | ✅ `StatusDot` + `disabled` pill |
| O3 | Three statistics: total, enabled, with custom domain | ✅ four — disabled is broken out |
| O4 | Filters: All, Enabled, Disabled | ✅ plus `Without domain` |
| O5 | Search covers name, alias, domain, description | ⚠️ narrowed to name, alias, domain — the admin screen's `searchIn`; description is shown as a column and remains sortable |
| O6 | Sorting by most recent / name / alias | ⚠️ replaced by column sorting |
| O7 | Empty state distinguishes "none yet" from "no match" | ✅ |
| O8 | Clicking a card opens the organization | ✅ → `…/organizations/:organizationId/settings` |
| O9 | Primary action `Create organization` | ✅ `New organization` |

## `ui/page-organization-detail.tsx` → mounts `iam/organization/ui/page-organization-detail.tsx`

| # | Rule | Carried over |
|---|---|---|
| OD1 | Fields: name (required), alias (required), domain, description, enabled | ✅ settings tab |
| OD2 | Alias constrained to `^[a-z0-9_-]+$`, error `Only lowercase letters, numbers, hyphens and underscores.` | ✅ through `updateOrganizationSchema` — the message is the schema's |
| OD3 | Save disabled unless dirty, alias valid and name non-empty | ✅ SaveBar on `isDirty`, `handleSave` returns early on a parse failure |
| OD4 | Reset restores every field | ✅ `Discard` |
| OD5 | Empty domain / description are sent as `null`, not `''` | ✅ `|| null` preserved |
| OD6 | Header shows name, `@alias` and a `Disabled` badge | ✅ pills |
| OD7 | Three stats: members, custom domain, status | ⚠️ dropped — a "1 / 0 custom domain" tile is a boolean dressed as a metric; the domain and the status are pills in the header, and the member count is the Members tab |
| OD8 | Account info: organization id, realm id, created, last update | ⚠️ realm id dropped — the console is already scoped to one realm and the shell names it. Id and created date stay in the header (FK-35) |
| OD9 | Delete asks for an in-place confirmation, then redirects to the list | ✅ danger zone → console list |
| OD10 | Member count read from the members endpoint | ✅ Members tab lists them |
| — | New: attributes tab, members tab (add / remove / organization-scoped roles), `redirect_url` field | gained from the admin screen |
| — | Groups tab **not** mounted | deliberate: groups are admin configuration, absent from the console's navigation, and its feature lives in `front/src/pages/`, which the console may not depend on |
| — | Tab state is a URL segment | ✅ FK-16 |

## `ui/page-roles.tsx` → mounts `iam/role/ui/page-roles-overview.tsx`

| # | Rule | Carried over |
|---|---|---|
| R1 | Scope badge: `client_id` present → client, absent → realm | ✅ |
| R2 | Four statistics: total, realm-level, client-level, distinct permissions | ⚠️ the fourth becomes `With permissions` (roles granting at least one) — a count of distinct permission strings across roles measures the catalogue, not the realm |
| R3 | Permission preview: first two chips, then `+N` | ⚠️ replaced by the permission count, right-aligned — same information, one column instead of a wrapping chip row |
| R4 | Filters: All, Realm-level, Client-level | ✅ plus `Without permissions` |
| R5 | Search covers name, description **and permission strings** | ⚠️ narrowed to name and description, as on the admin listing |
| R6 | Sorting by most recent / name / most permissions | ⚠️ replaced by column sorting |
| R7 | Row subtitle falls back to `No description` | ✅ falls back to `role_id: <id>` — FK-32, the absence is named with the identifier |
| R8 | Footer `Showing X of Y` and `N realm · N client-level` | ✅ `aggregates` |
| R9 | Clicking a row opens the role | ✅ → `…/roles/:role_id/settings` |

## `ui/page-role-detail.tsx` → mounts `iam/role/ui/page-role-detail.tsx`

| # | Rule | Carried over |
|---|---|---|
| RD1 | Name required (`Name is required`), description optional | ✅ `updateRoleSchema` |
| RD2 | Permissions grouped into eight labelled families | ✅ the admin permission catalogue, same families |
| RD3 | Save disabled unless dirty and valid | ✅ SaveBar on `dirtyCount`, `save()` returns early on a name error |
| RD4 | Definition and permissions are two separate mutations, each fired only if it changed | ✅ verbatim: `updateRole` and `updateRolePermissions` guarded by their own dirty flag |
| RD5 | Reset restores name, description and permissions | ✅ `Discard` |
| RD6 | Header states the scope and the granted permission count | ✅ pills |
| RD7 | Account info: role id, realm id, client id when client-scoped, created, last update | ⚠️ realm id dropped (single-realm console); role id and created date are in the header |
| RD8 | Delete asks for an in-place confirmation, then redirects to the list | ✅ danger zone → console list |
| — | Tab renamed `Users` → `Identities` | the only console-vocabulary change on this screen; the tab key stays `users`, so the URL segment is unchanged |
| — | Tab state is a URL segment | ✅ FK-16 |

## `ui/page-create-role.tsx` → mounts `iam/role/ui/page-create-role.tsx`

| # | Rule | Carried over |
|---|---|---|
| CR1 | Name required, description optional | ✅ `createRoleSchema` |
| CR2 | Four permission templates: Viewer, Editor, Admin, Custom | ⚠️ dropped — see below |
| CR3 | Custom requires at least one permission (`Pick at least one permission.`) | ⚠️ dropped with CR2; the admin screen accepts a role with no permission and the listing flags it afterwards |
| CR4 | The summary line counts the permissions that will be granted | ⚠️ dropped; the SaveBar states the creation instead |
| CR5 | Cancel returns to the role list | ✅ to the console list |
| — | New: realm / client scope choice, final at creation | gained from the admin screen; the CIAM creation could only make realm roles |

**On the dropped presets (CR2–CR4).** `Admin` was `Object.values(Permissions)`
— a one-click grant of every permission in the realm, including
`manage_realm`, offered from a customer console, with no confirmation. `Viewer`
and `Editor` were hand-maintained lists that drift from the enum the moment a
permission is added. The admin creation screen mounts the same permission
catalogue as the role detail, so the same role is reachable in two clicks more
and stays correct as the enum grows. Reported here rather than reproduced.

## UI ↔ domain divergences found

| Where | What the UI says | What the domain says |
|---|---|---|
| `ui/page-identities.tsx` | The `unverified` filter kept **every** account whose `email_verified` is false, including those with no email at all — an account with no address was permanently listed as "unverified" | `core/src/domain/user/entities.rs` — `email` is optional and `email_verified` defaults to false, so "no email" and "email not verified" are two different states. The admin listing excludes service accounts from its unverified count for the same reason. Carried over as-is (finding, not a fix) |
| `ui/page-identity-detail.tsx` | The credentials list labels every credential `Active` | `CredentialOverview` carries no status field. The label is unconditional decoration |
| `ui/page-create-role.tsx` | `ADMIN_PERMS = Object.values(Permissions)` presented as "Full access including realm and identity provider configuration" | `core/src/domain/role/entities.rs` — the permission bitmask is the authorization surface of the whole realm; the label understates what the preset grants |
| `ui/page-organization-detail.tsx` and `ui/page-create-organization.tsx` | Neither screen exposes `redirect_url` | `Organization` carries `redirect_url`, and the API accepts it on create and update. The CIAM screens silently dropped it on every save |
