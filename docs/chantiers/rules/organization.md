# Business rules inventory — `organization`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Source screens: the six `ui/` files of `front/src/pages/organization/`, the
layout, the schemas, and — for the rules that live nowhere else — the
features.

## `ui/page-organizations-overview.tsx`

| # | Rule | Carried over |
|---|---|---|
| G1 | Title `Organizations`, description naming what the resource is | ✅ description rewritten to say what an organization *does* (FK-22) |
| G2 | Primary action creates, and goes to `<organizations>/create` | ✅ |
| G3 | Search covers `name` and `alias` | ✅ **extended** to `domain` as well, as in the prototype — a superset, no row becomes unreachable |
| G4 | The list header counts the rows | ✅ table footer aggregate `<n> organizations` (FK's data-first aggregates) |
| G5 | Empty state `No organizations found in this realm.` | ✅ split in two per FK-13: "No organization" + creation, versus "No match" + clear-filter *and* creation (handled by `ListingPage`) |
| G6 | Clicking a row opens the organization's **settings** tab | ✅ `getHref` |
| G7 | A row shows: name, alias in mono, domain when present, `enabled`/`disabled` badge | ✅ same five columns; a missing domain reads `no domain` (FK-32) |
| G8 | Loading renders the list skeleton, not an empty state | ✅ `ListingPage loading` |

Added from the prototype: four metrics (Total, Enabled, Disabled, With a
domain) and an anomaly band. **No sparkline and no delta** — nothing in the
API exposes any history for organizations (FK-12); `MetricsBand` therefore
renders its counts variant.

The prototype's `Members` and `Groups` columns, its member/group metrics and
its "organization without a group" alert are **dropped**: the real
`Organization` (`api.client.ts`, `libs/ferriskey-organization/src/entities.rs`)
carries no counter, and `ListOrganizationsResponse` returns no aggregate.
Showing them would mean one members request and one groups request per row.

## `layout/organization-layout.tsx`

| # | Rule | Carried over |
|---|---|---|
| G9 | Back link returns to the organizations listing | ✅ |
| G10 | Title falls back `name ?? alias ?? '—'` | ✅ `name` (non-optional in the schema); the not-found case has its own screen |
| G11 | Sub-line: alias in mono, then the domain | ✅ moved to pills under the title |
| G12 | `enabled` / `disabled` badge in the header | ✅ `Pill` + `StatusDot` (FK-30: shape and colour) |
| G13 | Four tabs — Settings, Attributes, Members, Groups — matched by `startsWith` | ✅ `useRouteTabs`, same order (FK-16) |
| G14 | The `Add member` action appears only on the Members tab | ✅ the picker lives inside the Members tab |
| G15 | Creation date and internal id were **not** displayed | ✅ added in the header, right-aligned (FK-11) |

## `ui/page-create-organization.tsx` + `schemas/create-organization.schema.ts`

| # | Rule | Carried over |
|---|---|---|
| G16 | `name` required | ✅ schema reused verbatim |
| G17 | `alias` required and `^[a-z0-9_-]+$` | ✅ schema reused verbatim, message unchanged |
| G18 | `domain`, `redirectUrl`, `description` optional | ✅ |
| G19 | Defaults: `enabled = true`, everything else empty | ✅ |
| G20 | Validation on change; the action bar shows only when the form is valid | ✅ `safeParse` at render, as in the `role` pilot |
| G21 | Field order: Name, Alias, Enabled, Domain, Redirect URL, Description | ✅ order preserved, split in two sections (Definition / Contact & routing) to mirror the settings screen — FK-25, what is typed first comes first |
| G22 | Empty optional strings are sent as `null` | ✅ |
| G23 | On success, go to the new organization's settings tab | ✅ |
| G24 | On error, toast `Failed to create organization` | ✅ verbatim |
| G25 | Cancel returns to the listing | ✅ |
| G26 | The switch's label reads `Enabled` / `Disabled` | ✅ `SwitchField` (FK-27 — a real boolean keeps its switch) |

## `ui/page-organization-settings.tsx` + `schemas/update-organization.schema.ts`

| # | Rule | Carried over |
|---|---|---|
| G27 | The form is reset from the fetched organization | ✅ derived at render (`draft.key !== organization.id`), no `setState` in an effect |
| G28 | The save bar appears only when the form differs from the organization | ✅ same six fields compared |
| G29 | Cancel resets the form to the fetched values | ✅ `Discard` |
| G30 | Update sends all six fields, empty strings as `null` | ✅ |
| G31 | Two sections: General Settings (name, alias, enabled) then Contact & Routing (domain, redirect URL, description) | ✅ same split, same order |
| G32 | Every field carries a description | ✅ rewritten per FK-22 to state the consequence (the alias one now says renaming breaks existing links) |
| G33 | `name` required, `alias` required + same regex | ✅ schema reused verbatim |
| G34 | Danger zone last, confirmation quoting the organization name | ✅ `DangerZone` unchanged |
| G35 | Delete then navigate to the listing | ✅ |
| G36 | Loading renders a skeleton; a missing organization renders nothing | ✅ skeleton kept; the blank screen becomes a named "Organization not found" screen (FK-32, `role` pilot precedent) |

## `ui/page-organization-attributes.tsx`

| # | Rule | Carried over |
|---|---|---|
| G37 | The heading counts the attributes | ✅ `Attributes (n)` |
| G38 | `Add attribute` is hidden while the add row is open | ✅ |
| G39 | The add row saves only when key **and** value are non-empty once trimmed | ✅ `disabled` on the save button |
| G40 | Add row: `Enter` saves, `Escape` cancels (both fields) | ✅ |
| G41 | Values are trimmed before being sent | ✅ |
| G42 | Inline edit of the value only — the key is never editable | ✅ (the API keys the upsert on the path segment) |
| G43 | Edit row: `Enter` saves, `Escape` restores the original value | ✅ |
| G44 | An edit saving an empty value closes the row **without** saving | ✅ predicate copied verbatim |
| G45 | Delete acts immediately, without confirmation | ✅ unchanged (the toast comes from the mutation) |
| G46 | Loading renders four skeleton rows | ✅ |
| G47 | Empty state `No attributes defined for this organization.` | ✅ reworded to the singular register of the console |
| G48 | A column header row: Key, Value | ✅ |
| G49 | The key column is fixed-width and truncates | ✅ `w-48` |

## `ui/page-organization-members.tsx` + `ui/modals/add-member-modal.tsx`

| # | Rule | Carried over |
|---|---|---|
| G50 | Members are the `user_id`s of `GET …/members` resolved through the realm's user list; an unresolved id is dropped | ✅ copied verbatim |
| G51 | Search covers `username`, `email`, `firstname`, `lastname` | ✅ |
| G52 | Display name: service account → `Service Account`, else `firstname lastname`, else `username` | ✅ factored into `member-name.ts` |
| G53 | A service account is a user carrying a `client_id` (`isServiceAccount`) | ✅ imported, not reimplemented |
| G54 | Type badge: `service account` / `user account` | ✅ `Pill` violet / info |
| G55 | Status badge: `ACTIVE` / `INACTIVE` | ✅ `StatusDot` + `active` / `inactive` |
| G56 | A per-member action opens the role-mapping modal | ✅ the existing `ManageMemberRolesModalFeature` is mounted as-is |
| G57 | Removing asks for confirmation, and the text states that org-scoped roles are revoked and the user is **not** deleted | ✅ string copied verbatim |
| G58 | Empty state `This organization has no members yet.` | ✅ |
| G59 | Only users who are not already members can be added | ✅ same set difference, fed to `EntityPicker` (FK-28) |
| G60 | At least one user must be selected before adding | ✅ the add button only exists once something is staged |
| G61 | Adding issues one `POST …/members` per selected user | ✅ same loop |
| G62 | The picker is cleared after the add | ✅ |
| G63 | `addMemberSchema` (`userIds` min 1) | ⚠️ **not carried.** The zod schema only expressed "the array is not empty", which `EntityPicker` makes structurally impossible — there is no free-text path that could produce an invalid id. The rule survives as G60. |
| G64 | The add-member dialog and its `DataTable` with checkbox selection | ⚠️ **replaced** by `EntityPicker` per FK-28: the mission's explicit instruction, and the picker excludes the entries already retained by construction. |

## `feature/page-organization-groups-feature.tsx`

This screen has **no `ui/` file**: its 774 lines mix the tree, four
sub-tabs, three dialogs, pagination and their queries in one feature.

**Carried over by mounting the existing component inside the new tab.**
Same precedent as the email and portal builders in the chantier's decision
log: the chrome around it is restyled (page header, URL-anchored tabs,
sections), its implementation is untouched. Rewriting it would mean porting
a tree widget, a debounced paginated member list, a role multi-selector, an
attribute table and two confirmation dialogs — a workstream of its own, and
none of its rules would survive a hurried transcription. It is therefore
**visually inconsistent with the rest of the new console**, and knowingly so.

Its rules are consequently untouched, including the two defects listed
below.

## Divergences UI ↔ domain — reported, not fixed

1. **The front imposes an alias format the API does not, and misses the
   constraint the API does impose.** `create-organization.schema.ts` and
   `update-organization.schema.ts` require `^[a-z0-9_-]+$`;
   `libs/ferriskey-api-organization/src/validators.rs` only requires
   `length(min = 1)`. Conversely
   `libs/ferriskey-organization/src/services.rs:123` and `:204` reject a
   **duplicate alias within the realm**
   (`exists_organization_by_realm_and_alias`), which no screen anticipates:
   the clash surfaces as an error toast after the click. FK-39 asks for the
   opposite. Not fixed — the listing is loaded on the previous screen but
   not on the create screen, so anticipating it means a new query.
2. **`user.enabled ?? true` in the current members list.** `User.enabled` is
   a non-optional `boolean` in `api.client.ts`; the fallback can never fire.
   Dead defensive code, dropped in the new view without behaviour change.
3. **`isError` is accepted by `PageOrganizationMembers` and never used.** A
   failed member query renders as "this organization has no member". The
   new view has the same hole — surfacing it would mean an error state the
   feature does not currently produce.
4. **Adding members is not atomic and reports nothing.** One `POST` per
   user, no aggregation of the outcomes: a partial failure leaves the dialog
   closed with half the members added. The group version of the same gesture
   (`page-organization-groups-feature.tsx`) does handle it with
   `Promise.allSettled` and a summary toast. Behaviour carried over
   unchanged.
5. **French strings inside the groups feature.** `Precedent`, `Suivant`,
   `<from>-<to> sur <total>` in the members pagination of an otherwise
   English console.
6. **Organizations own no counters.** The domain entity has exactly
   `id, realm_id, name, alias, domain, redirect_url, description, enabled,
   created_at, updated_at`. Every member/group figure the prototype shows is
   fictional; see G7.
