# Business rules inventory — `user`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Sources read in full: `front/src/pages/user/ui/*` (10 files),
`front/src/pages/user/layouts/*`, `front/src/pages/user/feature/*`,
`front/src/pages/user/columns/*`, `front/src/pages/user/schemas/*`,
`front/src/pages/user/validators.ts`.

## `ui/page-users-overview.tsx` + `feature/page-users-overview-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| U1 | A user is a *service account* iff `client_id` is set (`isServiceAccount`) | ✅ reused verbatim from `@/utils` |
| U2 | Quick type filter with three states: All / Users / Service Accounts | ✅ `ListingPage.filters` — `All` is injected by the kit, so `Users` and `Service accounts` are declared; a third filter `Unverified email` is added, taken from the prototype's filter set and mapped onto a real field |
| U3 | Four statistics: total, enabled, disabled, email-verified | ✅ `metrics` |
| U4 | The *enabled* statistic shows `<pct>% active` only when both counts > 0, otherwise the literal `No enabled users` | ✅ `hint` |
| U5 | Search covers `username`, `email`, `firstname`, `lastname` | ✅ `searchIn` |
| U6 | Display name: service account → the literal `Service Account`; otherwise `firstname lastname`, falling back to `username` when both are empty | ✅ |
| U7 | Row subtitle: `email` falling back to `username` | ✅ card `subtitle` / `footer` |
| U8 | Status badge reads `enabled ?? true` — an undefined `enabled` is shown as active | ⚠ carried as `u.enabled` alone: `Schemas.User.enabled` is `boolean`, so the fallback is unreachable and hid the only case it could have covered (see divergence 1). No observable change |
| U9 | Type badge: `service account` / `user account` | ✅ `Pill` violet / info (FK-02) |
| U10 | Clicking a row opens the user's **overview** tab, not the bare detail URL | ✅ `getHref` |
| U11 | Loading renders skeleton rows, not an empty state | ✅ `ListingPage loading` |
| U12 | Empty state: `No users found.` | ✅ reworded `No user` + hint (FK-13) |
| U13 | The email-verified warning is suppressed for service accounts (`!user.email_verified && !user.client_id`, from `columns/list-user.column.tsx`) | ✅ carried into the email column of the new listing |
| U14 | Bulk delete quotes the count (`${count} users deleted`) and row delete confirms with the username | ⛔ **not carried** — already dead code: `PageUsersOverview` never calls `onRowDelete` nor `handleDeleteSelected`, and `DataView` exposes no row action. Deletion stays where it is reachable: the detail Danger Zone (U33–U34). Reported, not re-added |
| U15 | `filters` / `filterFields` / `onFiltersChange` were passed but unused (`filterFields` is a literal `[]`) | ⛔ dead props dropped with the view that ignored them |

## `layouts/users-layout.tsx`

| # | Rule | Carried over |
|---|---|---|
| U16 | Four sibling tabs (Clients, Users, Roles, Client Scopes) above the listing | ⛔ **not carried** — the `/next` shell sidebar carries that navigation; the pilot (`role`) dropped it identically |
| U17 | Primary action `New User` → `${USERS_URL}/create` | ✅ `New user` → `${NEXT_USERS_URL}/create` |

## `layouts/user-layout.tsx`

| # | Rule | Carried over |
|---|---|---|
| U18 | Back link returns to the users listing | ✅ |
| U19 | Header title is the `username`; subtitle is `firstname lastname`, plus ` · email` only when an email exists | ✅ subtitle recomposed as pills + a `dl` (FK-11) |
| U20 | Header carries a `verified` / `unverified` badge | ✅ `Pill` info / amber |
| U21 | Five tabs in this order: Overview, Credentials, Role Mapping, Organizations, Attributes | ✅ `useRouteTabs` — was `useState`-free but URL-driven by `pathname.includes`, now real segments (FK-16) |
| U22 | The *Add a role* action only exists on the role-mapping tab; *Add to organization* only on the organizations tab | ✅ each action now lives inside its own tab's `Section` header |

## `ui/page-user-overview.tsx`

| # | Rule | Carried over |
|---|---|---|
| U23 | `id` and `created_at` are read-only | ✅ moved out of the form into the header (FK-11) |
| U24 | `created_at` is formatted `en-US` numeric `MM/DD/YYYY` | ✅ preserved verbatim |
| U25 | `username` is rendered but **disabled** on update | ✅ disabled *and* justified (FK-23) — `UpdateUserValidator` has no `username` field, so the API cannot change it |
| U26 | Editable: `email`, `firstname`, `lastname`, `enabled`, `email_verified`, `required_actions` | ✅ |
| U27 | `required_actions` options come from the four `RequiredAction` values, humanised (`configure_otp` → `Configure Otp`) | ✅ options kept, labels rewritten as human labels + the raw key in mono (FK-04, FK-35); the humaniser is no longer needed |
| U28 | `enabled` switch label toggles `Enabled` / `Disabled`; `email_verified` toggles `Verified` / `Not verified` | ✅ `SwitchField onLabel/offLabel` (FK-27 — both are genuine on/off capabilities) |
| U29 | Validation: `username` required, `email` must be a valid address **or** the empty string | ✅ `updateUserValidator` reused verbatim |
| U30 | The save bar appears only when the form differs from the loaded user | ✅ dirty count derived at render (no `setState` in effect) |
| U31 | Cancel resets the form to the loaded user | ✅ `onDiscard` |
| U32 | Save toasts `User was updated`, errors toast `error.message` | ✅ |
| U33 | Danger Zone: delete quotes the username and warns about sessions, credentials and role assignments | ✅ `DangerZone` unchanged |
| U34 | Delete goes through `useBulkDeleteUser` with a single id, then navigates back to the listing | ✅ |
| U35 | The whole page renders `null` while loading or without a user | ✅ replaced by a skeleton and a *not found* panel, like the pilot |

## `ui/page-credentials.tsx` + `feature/page-credential-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| U36 | Four statistics: total, passwords (`credential_type === 'password'`), TOTP (`'otp'`), other (`total - passwords - totps`) | ✅ `MetricsBand` |
| U37 | The *passwords* statistic shows `<pct>% of total` only when both > 0, else `No password credentials` | ✅ `hint` |
| U38 | Row title is `credential_type`, capitalised, with a type badge coloured per type (`password` emerald, `otp` blue, `recovery_code` purple, anything else neutral) | ✅ `IconTile` + `Pill` tones, same three known types + neutral fallback |
| U39 | Row subtitle is `user_label` when present, otherwise `created_at` formatted `en-US` with `year/month/day/hour/minute` | ✅ preserved verbatim (FK-32 — the absence is replaced by the date, not left empty) |
| U40 | A *Reset password* action appears on a row **only** when `credential_type === 'password'` | ✅ becomes the section title switch (`Reset password` / `Set a password`) — the form is one per page, not one per row (see deviations) |
| U41 | Deleting a credential confirms with a dialog quoting the credential type | ✅ `ConfirmDeleteAlert` via `useConfirmDeleteAlert` |
| U42 | Deletion toasts `Credential was deleted` | ✅ moved into the mutation's `onSuccess` so the toast follows the result, not the click |
| U43 | Search covers `credential_type` and `user_label` | ✅ local filter on the same two keys |
| U44 | Loading renders a heading plus the literal `Loading...` | ✅ skeleton rows instead |

## `ui/modals/set-password.tsx` + `feature/modals/set-password-feature.tsx` + `schemas/index.ts`

| # | Rule | Carried over |
|---|---|---|
| U45 | The password schema is **built from the realm's public password policy** (`buildSetCredentialPasswordSchema(policy)`), not hardcoded | ✅ same builder, same hook (`usePublicPasswordPolicy`) |
| U46 | `confirmPassword` must equal `password`, error `Passwords must match` on the confirm field | ✅ |
| U47 | Entropy and common-password checks are server-side and surface as API errors | ✅ unchanged — the API error is toasted |
| U48 | `temporary` is submitted as part of the reset payload with `credential_type: 'password'` | ✅ |
| U49 | `temporary` was a `Switch` labelled *Temporary* | ✅ **converted to `ChoiceCards`** — temporary vs permanent are two natures of equal rank, not an on/off capability (FK-26) |
| U50 | Success toasts `Password has been set successfully`, resets the form, closes; failure toasts `Failed to set password` | ✅ same strings, form cleared instead of a modal closing |
| U51 | Missing `user_id` / `realm_name` toasts `User ID or Realm Name is missing` | ✅ |
| U52 | The password is write-only: nothing is ever read back | ✅ FK-34 — the field is empty on load, the description says the stored secret is never displayed |

## `ui/page-user-role-mapping.tsx` + `ui/modals/role-mapping-modal.tsx`

| # | Rule | Carried over |
|---|---|---|
| U53 | Four statistics: assigned, realm roles (`!client_id`), client roles (`client_id`), roles with at least one permission | ✅ |
| U54 | The *realm roles* statistic shows `<pct>% of total` only when both > 0, else `No realm roles` | ✅ |
| U55 | Scope badge: `client_id` present → `client`, absent → `realm` | ✅ `Pill` violet / info |
| U56 | Row subtitle falls back, in this order: `description`, then `client: <client_id>`, then `role_id: <id>` | ✅ preserved verbatim (FK-32) |
| U57 | Unassign calls `useUnassignUserRole` and toasts `Role unassigned successfully` with the description `The role has been successfully removed from the user.` | ✅ toast moved to the mutation callback (no `useEffect` on `isSuccess`) |
| U58 | The assign dialog only offers roles the user does **not** already hold | ✅ predicate copied |
| U59 | At least one role must be selected to assign (`assignRoleSchema`) | ✅ schema reused; the Assign button is disabled while the selection is empty |
| U60 | Assignment loops one call per selected role, then resets and closes | ✅ same loop |
| U61 | Success toasts `Role(s) assigned successfully` | ✅ |
| U62 | Role selection was a `DataTable` with checkboxes inside a modal | ✅ **replaced by `EntityPicker`** — `role_id` is a foreign key (FK-28) |
| U63 | Search covers role `name` and `description` | ✅ |
| U64 | Empty state: `No roles assigned to this user.` | ✅ reworded to name the consequence (FK-32/FK-22) |

## `ui/page-user-organizations.tsx` + `ui/modals/assign-organization-modal.tsx`

| # | Rule | Carried over |
|---|---|---|
| U65 | Assigned organizations are the intersection of `ListUserOrganizations` (memberships) and the realm's organizations, resolved by `organization_id`; unresolvable memberships are dropped | ✅ mapping copied verbatim |
| U66 | Four statistics: total, enabled, disabled, with a domain | ✅ |
| U67 | The *enabled* statistic shows `<pct>% of total` only when both > 0, else `No enabled organizations` | ✅ |
| U68 | Row shows `name`, `alias` in mono, and `domain` falling back to `org_id: <id>` | ✅ (FK-32) |
| U69 | Row carries an `enabled` / `disabled` badge | ✅ `Pill` — a disabled organization stays attached, the badge says it stopped acting |
| U70 | Remove calls `useRemoveUserFromOrganization`; the toast lives in the API layer | ✅ untouched |
| U71 | The assign dialog only offers organizations not already assigned | ✅ predicate copied |
| U72 | At least one organization must be selected (`assignOrganizationSchema`) | ✅ schema reused |
| U73 | Assignment loops one call per organization, resets and closes; success toasts `Organization(s) assigned successfully` | ✅ the per-call toast already emitted by `useAddUserToOrganization` is kept, and the aggregate toast is emitted once after the loop |
| U74 | Organization selection was a `DataTable` with checkboxes | ✅ **replaced by `EntityPicker`** — `organization_id` is a foreign key (FK-28) |
| U75 | Search covers `name`, `alias`, `domain` | ✅ |
| U76 | `isError` was accepted by the view and never used | ⛔ dropped with the prop; the tab now shows a load-failure line instead |
| U77 | Empty state: `No organizations assigned to this user.` | ✅ |

## `ui/page-user-attributes.tsx`

| # | Rule | Carried over |
|---|---|---|
| U78 | Heading shows the attribute count | ✅ `Section description` |
| U79 | A row edits in place; `Enter` saves, `Escape` cancels | ✅ |
| U80 | Saving trims the value and **skips the mutation when the trimmed value is empty**, but still leaves edit mode | ✅ predicate copied verbatim |
| U81 | Cancelling restores the original value | ✅ |
| U82 | Adding requires a non-empty trimmed key **and** a non-empty trimmed value | ✅ |
| U83 | The add row closes after a successful add; `Escape` cancels it | ✅ |
| U84 | Upsert sends a single-key `attributes` map (`{ [key]: value }`) — editing and adding are the same call | ✅ |
| U85 | Delete sends the `key`, not the id; the toast lives in the API layer | ✅ untouched |
| U86 | Loading renders three skeleton rows | ✅ |
| U87 | Empty state: `No attributes defined for this user.` | ✅ |
| U88 | Adding a key that already exists silently overwrites it | ✅ behaviour kept, and the duplicate is now named before the click (the prototype's `Cette clé existe déjà` becomes a warning, not a block — the API does upsert) |

## `ui/page-create-user.tsx` + `feature/page-create-user-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| U89 | Fields, in order: username, firstname, lastname, email, email_verified | ✅ same order (FK-25) |
| U90 | Validation: `username` required; `email` valid or empty; the rest optional | ✅ `createUserValidator` reused verbatim |
| U91 | The create bar appears only when the form is **valid and dirty** | ✅ same conjunction |
| U92 | `email_verified` switch labels `Verified` / `Unverified` | ✅ `SwitchField` (FK-27) |
| U93 | Success toasts `The user has been successfully created` and navigates to the listing | ✅ |
| U94 | Failure toasts `error.message` | ✅ |
| U95 | Cancel / back returns to the users listing | ✅ |

## Divergences UI ↔ domain — reported, not fixed

**1. `enabled` is treated as nullable by the listing, but is not.**
`ui/page-users-overview.tsx` renders `user.enabled ?? true`, and
`validators.ts` declares `enabled: z.boolean().optional()`. The generated
client says `enabled: boolean` (`api/api.client.ts`, `Schemas.User`), and the
Rust domain agrees — `core/src/domain/user/entities.rs` has a plain `bool`.
The `?? true` fallback can never fire; a user with `enabled === false` is
never mistaken for an active one. Kept as-is because removing it changes
nothing observable, but the optionality in `validators.ts` is what let the
fallback look necessary.

**2. `username` is editable in the prototype and in the update validator's
zod schema, but the API cannot change it.**
`Schemas.UpdateUserValidator` exposes `email`, `email_verified`, `enabled`,
`firstname`, `lastname`, `required_actions` — **no `username`**. The current
console correctly disables the field; `validators.ts` still declares
`username: z.string().min(1)` and sends it in the body, where it is ignored.
The prototype's `UserDetailPage` makes it a live `Input`. The `/next` screen
keeps it disabled and says why (FK-23). Not fixed: adding username updates is
an API change.

**3. `required_actions` is typed `Array<string>` on the way in and
`Array<RequiredAction>` on the way out.**
`Schemas.User.required_actions: Array<RequiredAction>` (four values) but
`Schemas.UpdateUserValidator.required_actions: Array<string> | null`. The
console can therefore submit an action the domain does not know. The `/next`
screen only offers the four domain values, which is the current behaviour too
(`Object.values(RequiredAction)`); the widening stays in the generated client.

**4. The user detail screen shows no session data.**
The prototype's `UserDetailPage` has a *Sessions* section (last sign-in,
active sessions, refresh tokens in circulation). None of it exists in the
domain: `Schemas.User` has no `last_sign_in_at`, and no endpoint under
`/realms/{realm_name}/users/{user_id}` returns sessions. The section is not
reproduced. Same for the prototype's listing column *Dernière connexion* and
its `aggregates` line counting users who never signed in.

**5. The prototype invents role inheritance.**
`user-detail/RoleMapping.tsx` distinguishes `direct` from `composite` roles
(`via <role>`) and refuses to unassign an inherited one. `GetUserRolesResponse`
is a flat `Array<Role>` with no source; composite roles do not exist in
`libs/ferriskey-domain/src/role/`. Every listed role is therefore treated as
directly assigned and removable, as the current console does.

**6. The prototype invents federated, read-only attributes and per-membership
organization roles/groups.**
`UserAttribute` has no `read_only` flag, and `OrganizationMember` is
`{ created_at, id, organization_id, user_id }` — no roles, no groups. Neither
is reproduced. `created_at` of the membership *is* available and is shown.

**7. `credential_type` is an open `string`, not an enum.**
`Schemas.CredentialOverview.credential_type: string`. The prototype types it
as a four-value union (`password | webauthn | otp | federated`) while the
current console styles three values (`password | otp | recovery_code`). The
`/next` screen keeps the three the console knows and falls through to a
neutral tone for anything else, which is what an open string requires.

**8. The listing's *Invite* action has no endpoint.**
The prototype's `UsersPage` offers a secondary `Inviter` button. No invitation
endpoint exists in `api.client.ts`. Not reproduced.
