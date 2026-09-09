# Business rules inventory — `client-scope`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Source screens: `front/src/pages/client-scope/` — `ui/` (6 pages + 2 modals),
`layout/` (2), `feature/` (6 + 2 modal features), `schemas/` (6),
`constants/protocol-mapper-templates.ts`, `components/`.

## `ui/page-client-scopes-overview.tsx`

| # | Rule | Carried over |
|---|---|---|
| CS1 | Four statistics: total scopes, default scopes, optional scopes, scopes with protocol mappers | ✅ `MetricsBand` via `ListingPage` |
| CS2 | Scope-type badge: `default_scope_type === 'DEFAULT'` → `default`, everything else → `optional` | ⚠️ carried over with a correction: the badge now renders the third domain value `none` instead of labelling it `optional`. See divergence D1 |
| CS3 | Each row shows the protocol as a badge | ✅ `Pill` neutral mono (FK-04) |
| CS4 | Row subtitle falls back to `scope_id: <id>` when `description` is empty | ✅ FK-32 |
| CS5 | Search covers `name`, `description`, `protocol` | ✅ `searchIn` |
| CS6 | List title carries the count; empty state `No client scopes found.` | ✅ count in the footer aggregate + `emptyLabel` / `emptyHint` / `emptyAction` (FK-13: two distinct empty states, provided by `ListingPage`) |
| CS7 | Clicking a row opens the scope's **details** tab | ✅ `getHref` |
| CS8 | Row delete opens a blocking dialog naming the scope and warning that protocol mappers and client mappings go with it | ✅ trailing action column + `ConfirmDeleteAlert`, same wording |
| CS9 | The delete control must not trigger the row navigation | ✅ the action column is not the first cell, so `DataView` does not wrap it in a `Link`; in **card** view the whole card is a link and no delete is offered — deletion there goes through the scope's Danger Zone. Reported deviation |
| CS10 | Delete buttons are disabled while the mutation is pending (`Deleting...`) | ⚠️ `ConfirmDeleteAlert` exposes no `disabled`; the confirmation is instead ignored while a deletion is in flight, so a double click cannot fire two requests |
| CS11 | Loading renders skeletons, not an empty state | ✅ `DataView loading` |

## `layout/client-scopes-layout.tsx`

| # | Rule | Carried over |
|---|---|---|
| CS12 | Four sibling tabs (Clients, Users, Roles, Client Scopes) with `startsWith` matching | ❌ **deliberately dropped** — the `/next` shell has one sidebar entry per domain (`next/shell/nav.ts`), so a second navigation row duplicates it. Same decision as the `role` pilot |
| CS13 | Page header `Client and Access Administration` / `Manage protocol mappers, scope mappings, and token attributes for client scopes.` | ⚠️ retitled `Client Scopes` with a description of the resource, since the header no longer covers four domains |
| CS14 | Primary action `New Client Scope` → `${CLIENT_SCOPES_URL}/create` | ✅ |

## `layout/client-scope-layout.tsx`

| # | Rule | Carried over |
|---|---|---|
| CS15 | Back link returns to the client-scopes listing | ✅ |
| CS16 | Header title falls back to `Client Scope` while loading; description falls back to `No description provided` | ✅ header renders nothing while loading (skeleton), and the description falls back to the mono `scope_id: <id>` (FK-32) |
| CS17 | Scope-type badge and protocol badge repeated in the detail header | ✅ |
| CS18 | Two tabs: Details, Protocol Mappers, `startsWith` matching | ✅ `useRouteTabs` (FK-16), segments `details` / `mappers` unchanged, label `Settings` for the first (prototype) |

## `ui/page-client-scope-detail.tsx`

| # | Rule | Carried over |
|---|---|---|
| CS19 | Loading renders a placeholder, not the form | ✅ skeleton |
| CS20 | `!scope` renders `Client scope not found.` | ✅ dedicated not-found panel |
| CS21 | Editable: `name` (required), `description` (optional) | ✅ |
| CS22 | `protocol` is displayed disabled, with `Only OpenID Connect is currently supported.` | ✅ FK-23 — the description now says *why* it is fixed |
| CS23 | Type is a two-option select: `Optional` / `Default` | ✅ + the consequence of each value under the control (FK-22), and a warning when the stored value is `NONE` (see D1) |
| CS24 | `Created At` and `Updated At` are read-only rows | ⚠️ moved out of the form into the header (FK-11). Format changed from `toLocaleString()` to `en-GB` date-time — the console is English |
| CS25 | Attributes section: empty → `No attributes configured.`, otherwise name / value rows with `-` for an empty value | ✅ (`—` for an empty value). See divergence D3: the API never populates it |
| CS26 | Protocol mappers listed in the details tab with name, `mapper_type` and id | ⚠️ dropped as a duplicate — the Protocol Mappers tab is the single place where mappers are listed, and it is richer. The scope's mapper count is in the header and on the tab |
| CS27 | Danger zone quoting the scope name, warning about mappers and client mappings | ✅ `DangerZone` unchanged |
| CS28 | The save bar appears only when the form is valid **and** dirty **and** no mutation is pending | ✅ `dirtyCount > 0 && !nameError` |
| CS29 | Save label becomes `Saving...` while pending | ✅ |
| CS30 | Cancel resets the form to the loaded scope | ✅ `Discard` |
| CS31 | Update payload: `description` trimmed, empty → `null`; `protocol` taken from the loaded scope; `is_default = scopeType === 'default'` | ✅ verbatim |
| CS32 | Deleting navigates back to the listing on success | ✅ |

## `ui/page-create-client-scope.tsx`

| # | Rule | Carried over |
|---|---|---|
| CS33 | Back control to the listing + a chip naming the current step | ✅ back button; the chip is replaced by the page title `New client scope` (FK page skeleton) |
| CS34 | Same four fields, `protocol` fixed to `openid-connect` and disabled | ✅ |
| CS35 | Type defaults to `optional` | ✅ |
| CS36 | `name` required, message `The client scope name is required` | ✅ `createClientScopeSchema` reused verbatim |
| CS37 | The create bar appears only when the form is valid, dirty and not pending | ✅ |
| CS38 | On success, navigate to the listing | ✅ |

## `ui/page-protocol-mappers.tsx`

| # | Rule | Carried over |
|---|---|---|
| CS39 | Three statistics: total mappers, role mappers (`mapper_type` contains `role`), identity mappers (`usermodel` \| `attribute` \| `property` \| `full-name`) | ✅ `MetricsBand` in the tab |
| CS40 | Role / identity statistics show `<pct>% of total` when both counts are > 0, otherwise `No role mappers` / `No identity mappers` | ✅ `hint` |
| CS41 | Category badge, tested in this exact order: `role`, `audience`, `hardcoded`, `attribute`, `organization`, `property`\|`full-name`\|`usermodel` → `identity`, else `custom` | ✅ order copied verbatim into `mapper-categories.ts`; tones remapped onto the kit palette (`organization` and `custom` share `neutral` — the kit has no seventh classification tone) |
| CS42 | Row shows the name, the category badge and the `mapper_type` in mono | ✅ + the claim name and the token destinations (prototype) |
| CS43 | Clicking a row opens the mapper settings page | ✅ `Configure` link on the row |
| CS44 | Delete goes through `ConfirmDeleteAlert`, quoting the mapper name, `This action cannot be undone.` | ✅ verbatim |
| CS45 | `Add Mapper` opens the template picker | ✅ |
| CS46 | Empty state `No protocol mappers configured.` | ✅ reworded to say what the absence means (FK-32) |
| CS47 | Search over `name` and `mapper_type` | ❌ **dropped** — the tab lists the mappers of a single scope inside a `Section`, not a `ListingPage`; a search field over a handful of rows costs more than it returns. Reported |

## `ui/page-create-protocol-mapper.tsx`

| # | Rule | Carried over |
|---|---|---|
| CS48 | The template is read from `?template=<id>`; an unknown id redirects to the mappers tab | ✅ FK-29 |
| CS49 | Banner with the template icon, name, description, and its `mapper_type` in mono — hidden when the template is `isCustom` | ✅ moved into the page header |
| CS50 | `name` required (`Name is required`) | ✅ `mapperTemplateFormSchema` reused |
| CS51 | The `Mapper Type` field appears **only** for the custom template | ✅ |
| CS52 | The configuration section renders only when the template has fields or is custom | ✅ |
| CS53 | Custom template → raw JSON textarea validated as JSON (`Config must be valid JSON`); otherwise the dynamic fields of the template | ✅ |
| CS54 | Defaults on entry: `name = template.defaultName`, `mapper_type = isCustom ? '' : template.mapper_type`, every config field at its `defaultValue` | ✅ derived at render, no `setState` in an effect |
| CS55 | Config values are coerced: `'true'` / `'false'` → booleans, everything else stays a string; unparsable JSON → `{}` | ✅ verbatim |
| CS56 | Submit is blocked while invalid or pending; label `Creating...` | ✅ `FloatingActionBar` shown only when submittable |
| CS57 | Cancel returns to the mappers tab | ✅ |
| CS58 | A custom template with an **empty** `mapper_type` is submittable (the schema marks it optional) | ⚠️ **deliberately tightened**: the create bar stays hidden until a custom mapper type is filled in. The prototype does the same (`disabled={!name \|\| !mapperType}`) and FK-39 asks that a domain refusal be announced before the click — a mapper with an empty type is not a thing the API can store usefully |

## `ui/page-protocol-mapper-settings.tsx`

| # | Rule | Carried over |
|---|---|---|
| CS59 | Back link `Protocol Mappers` | ✅ |
| CS60 | Banner: template icon or `⚙️`, template name or the mapper name, template description if any, `mapper_type` in mono | ✅ in the page header |
| CS61 | Read-only cards: mapper type, creation date (`fr-FR`) | ⚠️ moved to the header (FK-11); date formatted `en-GB` — the console is English |
| CS62 | Only `name` is editable; the mapper type is not | ✅ |
| CS63 | The template is matched by `mapper_type` among the non-custom templates | ✅ verbatim |
| CS64 | Template matched **and** it has fields → dynamic fields; no template → raw JSON textarea | ✅ |
| CS65 | The save bar shows when the form is dirty **or** the config values differ from the loaded ones | ✅ predicate copied |
| CS66 | Reset restores both the name and the config values | ✅ |
| CS67 | The update payload always resends `mapper.mapper_type` unchanged | ✅ |
| CS68 | Scope loaded but the mapper id is unknown → redirect to the mappers tab | ✅ rendered as a not-found panel with a back link instead of a render-phase `navigate()` (which is a React side effect during render) |

## `ui/modals/mapper-template-picker-modal.tsx`

| # | Rule | Carried over |
|---|---|---|
| CS69 | Two sections: `Quick Start` (cards grid) then `By configuration` (catalog rows), each with its subtitle | ✅ |
| CS70 | A catalog row hides the mono `mapper_type` for the custom template | ✅ |
| CS71 | Picking a template closes the modal and navigates to `mappers/new?template=<id>` | ✅ FK-29 |

## `ui/modals/edit-protocol-mapper-modal.tsx` + its feature

| # | Rule | Carried over |
|---|---|---|
| CS72 | Modal editing name, mapper type and raw JSON config of a mapper | ❌ **not migrated — dead code**. `EditProtocolMapperModalFeature` is imported by nothing (`grep` over `front/src`); the mapper settings page superseded it |

## FK-28 — foreign keys stop being free text

`constants/protocol-mapper-templates.ts` declares config fields that hold a
foreign key but are typed `text`:

| Config key | Template | Points at |
|---|---|---|
| `client.id` | User Client Role | a client's `client_id` |
| `included.client.audience` | Audience (catalog) | a client's `client_id` |
| `role` | Role Name Mapper, Hardcoded Role | a role name, or `clientId.roleName` |

Those three keys now render an `EntityPicker`-style single-value combobox fed
by `useGetClients` / `useGetRoles`, instead of a free-text input (FK-28). The
stored value stays exactly the string the API expects. A value that matches no
known entity is still displayed, flagged `unknown` (FK-32), and clearing is
allowed because every one of those fields documents an "leave empty" meaning.

## Lines needed in read-only files

`front/src/next/routes.ts` has no per-scope helper, the way it has
`NEXT_ROLE_URL`. Until it does, `next/pages/client-scope/urls.ts` composes the
URLs locally. The line to add:

```ts
export const NEXT_CLIENT_SCOPE_URL = (realmName = ':realm_name', scopeId = ':scope_id') =>
  `${NEXT_CLIENT_SCOPES_URL(realmName)}/${scopeId}`
```

## Divergences UI ↔ domain — reported, not fixed

**D1 — the domain has three scope types, the API can only write two.**

`libs/ferriskey-aegis/src/entities.rs:13` declares
`ScopeType { None, Optional, Default }` and `ClientScope::new` builds a scope
with `ScopeType::None`. But both write paths take a boolean:
`CreateClientScopeValidator.is_default` / `UpdateClientScopeValidator.is_default`,
and `core/src/infrastructure/aegis/repositories/client_scope_postgres_repository.rs:41`
and `:142` map it to `DEFAULT` when true and `OPTIONAL` when false. A scope
stored as `NONE` therefore cannot be written back as `NONE` — saving it from
the console silently promotes it to `OPTIONAL`. The current console hides this
entirely: its badge renders anything that is not `DEFAULT` as `optional`.
Not fixed: the fix is a new API shape (`scope_type` instead of `is_default`).
The migrated screens show the real value and warn before the save.

The prototype offers `none` as a third option in its `Type` select. That take
is not reproduced: the API cannot store it.

**D2 — a client scope does not know how many clients use it.**

The prototype's `assignedTo` field, its `Clients` column, its `Inutilisés`
metric and its "attached to N clients" alert have no counterpart in the domain.
`ClientScopeMapping { client_id, scope_id, default_scope_type }`
(`libs/ferriskey-aegis/src/entities.rs:145`) exists, but it is only ever read
per client (`GET /clients/{id}/client-scopes`); nothing exposes the reverse
index, and `Schemas.ClientScope` carries no count. The listing therefore
replaces that axis with the one the domain does expose: protocol mappers.

**D3 — `ClientScope.attributes` is always `null` over the wire.**

`core/src/infrastructure/aegis/mappers.rs:18` sets `attributes: None`, and
neither `get_client_scope` nor `get_client_scopes`
(`libs/ferriskey-aegis/src/services/client_scope_service.rs:120-191`) hydrates
it — they only fill `protocol_mappers`. A `ClientScopeAttributeRepository`
with a `get_attributes` method exists
(`core/src/infrastructure/aegis/repositories/client_scope_attribute_postgres_repository.rs:65`)
and is simply never called on the read path. The Attributes section of the
detail screen can consequently never show anything. It is kept — with a named
empty state — because the field is part of the public schema and the fix is one
line of service code, outside this chantier.

**D4 — the prototype is right about protocol mappers.**

It asserts that a `ClientScope` carries its mappers rather than a frozen count,
and the domain agrees: `ClientScope.protocol_mappers: Option<Vec<ProtocolMapper>>`
is hydrated on **both** read paths — per scope
(`client_scope_service.rs:143-147`) and for the whole realm
(`client_scope_service.rs:182-188`). The listing can therefore count and filter
on mappers client-side without an extra request, which is what the migrated
overview does. No divergence to report here beyond D2's absence of a client
count.
