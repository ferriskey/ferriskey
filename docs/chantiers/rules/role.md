# Business rules inventory — `role`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

## `ui/page-roles-overview.tsx`

| # | Rule | Carried over |
|---|---|---|
| R1 | Scope badge: `client_id` present → `client`, absent → `realm` | ✅ `Pill` violet / info (FK-02) |
| R2 | Permission count is pluralised (`1 permission` / `2 permissions`) | ✅ |
| R3 | Row subtitle falls back to `role_id: <id>` when `description` is empty | ✅ FK-32 — the absence is named |
| R4 | Search covers `name` and `description` only | ✅ `searchIn` |
| R5 | Four statistics: total, realm, client, with-permissions | ✅ `MetricsBand` |
| R6 | Realm-roles stat shows a percentage of total, but only when both > 0; otherwise the literal `No realm roles` | ✅ `hint` |
| R7 | Clicking a row opens the role's **settings** tab, not an overview | ✅ `getHref` |
| R8 | Delete goes through `ConfirmDeleteAlert` (blocking confirmation) | ✅ unchanged |
| R9 | Loading state renders skeleton rows, not an empty state | ✅ `DataView loading` |

Dead props kept in the signature because the feature still passes them and
the split forbids touching it: `filters`, `filterFields`, `onFiltersChange`,
`handleDeleteSelected`, `onRowDelete`, `realmName`. They were already unused
by the previous view — `handleDeleteSelected` and `onRowDelete` were never
wired to anything. Reported, not removed.

## `layout/roles-layout.tsx`

| # | Rule | Carried over |
|---|---|---|
| R10 | Four sibling tabs (Clients, Users, Roles, Client Scopes) with `startsWith` active matching | ✅ factored into `components/access-administration-header.tsx` |
| R11 | Clients / Users tabs target `/overview`; Client Scopes targets `CLIENT_SCOPES_OVERVIEW_URL` | ✅ preserved verbatim |
| R12 | Primary action `New Role` → `${ROLES_URL}/create` | ✅ |

## `layout/role-layout.tsx`

| # | Rule | Carried over |
|---|---|---|
| R13 | Back link returns to `${ROLES_URL}/overview`, not to `${ROLES_URL}` | ✅ |
| R14 | Header title falls back to nothing while loading; description falls back to `role_id: <id>` | ✅ |
| R15 | Scope badge and permission count repeated in the detail header | ✅ |
| R16 | Three tabs: Settings, Permissions, Users in role | ✅ `useRouteTabs` (FK-16) |

## `ui/page-role-settings.tsx`

| # | Rule | Carried over |
|---|---|---|
| R17 | Loading renders a skeleton | ✅ |
| R18 | `!role` renders a dedicated "not found" screen naming the realm | ✅ |
| R19 | Only `name` and `description` are editable | ✅ |
| R20 | Three read-only facts: permission count, client, creation date | ✅ moved to the header (FK-11 — immutable metadata stays out of the form) |
| R21 | Creation date is formatted `fr-FR` | ✅ preserved verbatim, see the deviation note in the file |
| R22 | The save bar only appears when `hasChanges` | ✅ `FloatingActionBar` unchanged |
| R23 | Cancel resets the form | ✅ |
| R24 | Delete carries a confirmation quoting the role name | ✅ `DangerZone` unchanged |

## `ui/page-role-permissions.tsx`

| # | Rule | Carried over |
|---|---|---|
| R25 | The save bar appears when the selected set differs from the role's, in **either** direction (length differs, or any selected permission is absent from the role) | ✅ predicate copied verbatim |
| R26 | Permission names are humanised: `manage_users` → `Manage Users` | ✅ |
| R27 | Each group shows `<enabled> of <total> permissions enabled` | ✅ |
| R28 | A permission tile toggles on click, anywhere on the tile | ✅ |

## `ui/page-create-role.tsx`

| # | Rule | Carried over |
|---|---|---|
| R29 | The `Client` field appears **only** when `roleScope === 'client'` | ✅ |
| R30 | `clientId` validation error is rendered under the picker | ✅ |
| R31 | The create bar appears only when `form.formState.isValid` | ✅ |
| R32 | A group checkbox selects/deselects the whole group | ✅ |
| R33 | Each group shows `<selected>/<total>` | ✅ |
| R34 | Selected permissions are listed as removable chips; empty → the literal `No permissions selected` | ✅ FK-32 |
| R35 | Each permission row carries a badge with its verb (`manage`, `view`, `query`…) | ✅ |
| R36 | Cancel returns to the roles listing | ✅ |

## Divergences UI ↔ domain — reported, not fixed

**The front can only grant 19 of the 27 permissions the domain defines.**

`libs/ferriskey-domain/src/role/permission.rs` declares 27 variants;
`front/src/api/core.interface.ts` mirrors only 19, and
`pages/role/types/permission-groups.ts` groups exactly those 19. Missing:

- `manage_webhooks`, `query_webhooks`, `view_webhooks` (bits 19–21)
- `manage_client_scopes`, `query_client_scopes`, `view_client_scopes` (22–24)
- `manage_email_templates`, `view_email_templates` (25–26)

An administrator therefore cannot grant webhook, client-scope or email-template
permissions through the console at all. Not fixed here: adding them changes what
a role can be granted, which is a behaviour change with real blast radius — it
belongs to its own change, not to a style migration.
