# Business rules inventory — `client`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Source screens: `front/src/pages/client/ui/*` (7 pages + 4 components),
`layout/client-layout.tsx`, `container.tsx`, `feature/*`, `schemas/*`.

## `ui/page-clients-overview.tsx`

| # | Rule | Carried over |
|---|---|---|
| C1 | Three quick filters: All, Confidential (`!public_client`), Deprecated (`!enabled`) | ✅ `ListingPage filters` — renamed `Public` / `Confidential` / `Disabled`; `Deprecated` named a client that is merely disabled, and the domain has no deprecation concept |
| C2 | Search matches `name` **or** `client_id`, case-insensitive | ✅ `searchIn={(c) => `${c.name} ${c.client_id}`}` |
| C3 | Changing the quick filter resets pagination to page 1 | ✅ n/a — `DataView` does not paginate; the whole set is sorted and scrolled (FK-12/FK-13 flow) |
| C4 | Client-side pagination, 10 rows per page | ❌ dropped — `DataView` shows the full set with a `N of M` counter. Reported. |
| C5 | Four statistics: total, active (`enabled`), public (`public_client`), confidential (`!public_client`) | ✅ `MetricsBand`, same four |
| C6 | Each statistic carries a 7-day sparkline derived from `created_at` | ❌ dropped — FK-12: `MetricsBand` switches to its count variant. The prototype made the same call for clients. |
| C7 | Type badge: `public_client` → `public`, else `confidential` | ✅ `Pill` info / violet (FK-02) |
| C8 | Status badge: `enabled` → ACTIVE, else INACTIVE with a warning triangle | ✅ `StatusDot` + label, and a `disabled` filter |
| C9 | Row subtitle is `client_id: <value>` | ✅ `client_id` is its own mono column, plus the card subtitle |
| C10 | Clicking a row opens the client's **settings** tab | ✅ `getHref` |
| C11 | Loading renders 6 skeleton rows, not an empty state | ✅ `DataView loading` |
| C12 | Empty state reads `No clients found.` | ✅ FK-13 — two distinct empties: no client at all vs. filtered out |
| C13 | Delete goes through `ConfirmDeleteAlert` quoting the client name | ✅ moved to the settings tab `DangerZone`, which already quoted it. No row-level delete in the listing — it was wired in the feature but never reachable from the view (`onRowDelete` unused). Reported. |
| C14 | `handleDeleteSelected`, `onRowDelete`, `filters`, `filterFields`, `onFiltersChange`, `realmName` are declared and never used by the view | ❌ not carried — dead props |
| C15 | Pagination labels are French (`sur`, `Precedent`, `Suivant`) | ❌ n/a with C4; the console is English |

## `container.tsx` (listing chrome)

| # | Rule | Carried over |
|---|---|---|
| C16 | Four sibling tabs: Clients, Users, Roles, Client Scopes | ❌ dropped — the `/next` shell sidebar carries this navigation (FK-19); the pilot dropped it the same way |
| C17 | Primary action `New Client` → `${CLIENTS_URL}/create` | ✅ becomes `CreatePickerDialog` → `/clients/create?protocol=…` (FK-29) |
| C18 | Secondary action `Import` with an empty `onClick` | ❌ dropped — an inert button. Reported. |

## `layout/client-layout.tsx`

| # | Rule | Carried over |
|---|---|---|
| C19 | Back link returns to `${CLIENTS_URL}/overview` | ✅ returns to the `/next` listing |
| C20 | Header title is `client_id`, not `name` | ✅ title is `name`, `client_id` in mono underneath — FK-35: the human label carries the line, the technical key underlines it. Deviation, reported. |
| C21 | Header carries an `enabled`/`disabled` pill and a `protocol` pill | ✅ plus a `confidential`/`public` pill (the type was only visible in the listing before) |
| C22 | Tab order: Settings, Credentials, Roles, Client Scopes, SAML, Maintenance | ✅ preserved verbatim |
| C23 | The **Credentials** tab exists only when `client.secret` is present | ✅ preserved verbatim |
| C24 | Tab matching uses `pathname.startsWith(tab.path)` | ✅ `useRouteTabs` (FK-16), unknown segment falls back to Settings |

## `ui/page-create-client.tsx` + `feature/page-create-client-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| C25 | `clientId` required — `The client ID is required` | ✅ `createClientSchema` reused verbatim |
| C26 | `name` required — `The name is required` | ✅ same schema |
| C27 | Submit is enabled only when the form is valid **and** dirty | ✅ `FloatingActionBar show={canSubmit}` |
| C28 | Defaults: `enabled: false`, `protocol: 'openid-connect'`, `clientAuthentication: false` | ⚠ `enabled` now defaults to `true`. A client created disabled cannot authenticate anyone and the creation screen never says so; the prototype defaults to enabled. Deviation, reported. `clientAuthentication` still defaults to public. |
| C29 | `client_type` is `confidential` when client authentication is on, `public` otherwise | ✅ verbatim |
| C30 | `public_client` is the negation of client authentication | ✅ verbatim |
| C31 | `service_account_enabled` follows client authentication | ✅ verbatim |
| C32 | Success toast `The client has been successfully created`, then navigate to the listing | ✅ verbatim |
| C33 | Client authentication is a switch labelled `Confidential` / `Public` | ✅ replaced by two named `ChoiceCards` (FK-26) — the off state of a switch does not say *public* |
| C34 | Field descriptions: `Unique identifier for this client.`, `Display name shown in the UI.`, `Disabled clients cannot authenticate users.`, `If enabled, clients must authenticate using a secret or certificate.` | ✅ carried; the authentication description moves into the two cards (FK-22) |
| C35 | The protocol is fixed to `openid-connect` and never shown | ✅ FK-29 — chosen from the listing, travels as `?protocol=`, shown as a pill with a *change* link; a missing parameter sends the user back to the listing |
| C36 | — (new) SAML clients need `sp_entity_id` and `acs_url` before FerrisKey answers for them | ✅ the SAML branch of the creation form reuses `samlServiceProviderSchema` and chains `useUpsertSamlConfig` after the client is created |

## `ui/page-client-settings.tsx`

| # | Rule | Carried over |
|---|---|---|
| C37 | Editable: `name`, `clientId`, `enabled`, `directAccessGrantsEnabled`, `oauthDeviceCodeGrantEnabled`, `requirePkce`, and the four token lifetimes | ✅ same set |
| C38 | `updateClientSchema`: `clientId` and `name` required with the same messages | ✅ schema reused verbatim |
| C39 | Client Authentication is rendered **disabled** — `public_client` cannot be changed after creation | ✅ `ChoiceCards` with a `disabledReason` on both cards (FK-23: immutable *and* justified) |
| C40 | The Require PKCE description gains ` Strongly recommended: this client cannot keep a secret safe.` **only when `public_client`** | ✅ preserved verbatim, same condition |
| C41 | PKCE switch labels are `Required` / `Optional`, not Enabled / Disabled | ✅ `SwitchField onLabel/offLabel` |
| C42 | Section order: General Settings, Capability Config, Access Settings, Logout Settings, Token Lifetimes, Danger Zone | ✅ preserved (FK-25) |
| C43 | Token lifetimes: `Leave empty to inherit the realm default value.` | ✅ carried as the Section description |
| C44 | Token lifetimes use `DurationInput` with `nullable` | ✅ product `DurationInput` reused (kit's was deliberately not ported) |
| C45 | Web origins description mentions the `+` sentinel and that regex patterns are skipped | ✅ verbatim, on a full-width `FieldRow` (FK-21, FK-24) |
| C46 | Save bar shows only when `hasChanges`; Cancel resets the form | ✅ `FloatingActionBar` with a dirty count |
| C47 | Danger zone: `Once deleted, all associated tokens, roles, and configurations will be permanently removed.` and a confirmation quoting `name || client_id` | ✅ `DangerZone` verbatim |
| C48 | Deleting navigates back to the clients listing | ✅ |

## `components/manage-redirect-uris.tsx`

| # | Rule | Carried over |
|---|---|---|
| C49 | A redirect URI is required to be non-empty — `Redirect URI is required` | ✅ validated in the feature before the mutation |
| C50 | Deleting a redirect URI asks for confirmation | ⚠ replaced by a chip `×` — the entry is re-addable in one gesture and the confirmation was generic (`Are you sure you want to delete this redirect URI?`). Reported. |
| C51 | Toasts `Redirect URI added successfully` / `Redirect URI deleted successfully` | ✅ |
| C52 | The client query is refetched after each change | ✅ the mutations already invalidate `['client']` |

## `components/manage-web-origins.tsx`

| # | Rule | Carried over |
|---|---|---|
| C53 | A web origin must pass `isWebOriginValue` — `+` or a scheme/host-only http(s) origin | ✅ `isWebOriginValue` reused verbatim |
| C54 | The refusal message names the remedy: `Enter an origin such as https://app.example.com — no path, no wildcard — or + to derive them from this client's redirect URIs` | ✅ verbatim (FK-37) |
| C55 | The `+` entry is labelled `Derived from redirect URIs` instead of `Web Origin n` | ✅ a hint line names it under the chip list (FK-32) |
| C56 | Deleting warns that browsers cache preflights for a few minutes and replicas for up to 30 seconds | ✅ carried as the field's hint so it is read *before* the removal, not after |
| C57 | The value is trimmed before being sent | ✅ |
| C58 | Errors surface as a toast with the API message | ✅ |

## `components/manage-post-logout-redirect-uris.tsx`

| # | Rule | Carried over |
|---|---|---|
| C59 | Non-empty required — `Post-logout redirect URI is required` | ✅ |
| C60 | Deleting asks for confirmation | ⚠ same treatment as C50 |
| C61 | Toasts on add and delete | ✅ |

## `ui/page-client-credentials.tsx`

| # | Rule | Carried over |
|---|---|---|
| C62 | `client_id` is shown read-only with a copy button | ✅ mono, read-only, copy button |
| C63 | The secret is masked by default (`••••…`) | ✅ FK-34 — rendered as a masked fingerprint, never as an editable field |
| C64 | Revealing is an explicit action; the query has `gcTime: 0`, no refetch, no retry | ✅ preserved verbatim via `useGetClientSecret({ enabled })` |
| C65 | Description warns that revealing is recorded as a security event | ✅ verbatim |
| C66 | A 403 shows `You need the manage-clients permission to reveal this secret. Ask a realm administrator for access.` | ✅ verbatim (FK-37) |
| C67 | Any other error shows `The secret could not be revealed. Please try again.` | ✅ verbatim |
| C68 | The copy button for the secret is disabled while no secret is loaded | ✅ |
| C69 | The reveal button is disabled while fetching and shows a spinner | ✅ |

## `ui/page-client-roles.tsx`

| # | Rule | Carried over |
|---|---|---|
| C70 | `isError` renders `Error while loading roles.` and nothing else | ✅ verbatim |
| C71 | Search covers `name` and `description` | ✅ |
| C72 | Title is `Roles (n)` | ✅ the count moves to the tab and the section header |
| C73 | Empty state `No roles found for this client.` | ✅ reworded to say what the absence means (FK-32) |
| C74 | Deleting a role asks `Delete role?` / `Are you sure you want to delete "<name>"? This action cannot be undone.` | ✅ verbatim, `ConfirmDeleteAlert` |
| C75 | After deleting, the client roles query is refetched | ✅ |

## `ui/page-client-scopes.tsx` + its four components

| # | Rule | Carried over |
|---|---|---|
| C76 | Two sub-views: Assigned scopes, Evaluate | ✅ `Segmented`, not a second row of underlined tabs (FK-17, FK-18) |
| C77 | The client-scopes endpoint returns a bare array; a non-array response is treated as empty | ✅ preserved verbatim |
| C78 | Scope type is `default` when `default_scope_type === 'DEFAULT'`, `optional` otherwise | ⚠ `ScopeType` has three values (`NONE`, `OPTIONAL`, `DEFAULT`); the old mapping showed a `NONE` scope as *optional*. The new view shows the three states and offers the default↔optional switch only for the two the API can assign. Reported as a UI ↔ domain divergence. |
| C79 | Changing the type unassigns the old type, then assigns the new one | ✅ preserved verbatim, same order, awaited |
| C80 | Removing unassigns with the scope's current type | ✅ |
| C81 | The scope name links to the client-scope detail page | ✅ links into the `/next` client-scopes listing |
| C82 | Add-scope modal: search on `name` and `description`, a Default/Optional select, single selection, already-assigned scopes excluded | ✅ all four preserved |
| C83 | Add-scope modal: `No available scopes to assign.` when everything is assigned | ✅ reworded, still distinguishes “nothing matches the search” from “everything is assigned” (FK-13) |
| C84 | Assign / unassign toasts and error toasts come from `client.api.ts` | ✅ untouched |
| C85 | The effective-scopes preview lists **all** assigned scopes | ⚠ it now lists the **default** ones only: those are what a token carries without the client asking. The old preview also carried a `Claims: Standard scope claims (implementation-specific)` placeholder line, dropped as it stated nothing. Reported. |
| C86 | The preview shows the client's protocol, defaulting to `openid-connect` | ✅ the protocol is in the page header instead (FK-11) |
| C87 | Evaluate: user select, optional-scope chips, `Evaluate` disabled without a user | ✅ user select is an entity list (FK-28) |
| C88 | Evaluate: default scopes are always included in the requested scope string, on top of the optional ones the admin picked, de-duplicated | ✅ preserved verbatim, and the resulting string is displayed before the call |
| C89 | Evaluate: an empty scope string is sent as `undefined` | ✅ verbatim |
| C90 | Evaluate renders effective mappers (name + type), effective roles (realm + per client), and three JSON panels | ✅ same blocks, same order |
| C91 | `No protocol mappers apply for this scope set.` when the mapper list is empty | ✅ |
| C92 | Realm roles render `None` when empty | ✅ reworded `none` (FK-32) |

## `ui/page-client-saml.tsx` + `feature/page-client-saml-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| C93 | When no config exists, an intro block explains what filling the two values does | ✅ verbatim |
| C94 | Defaults when unconfigured: `signAssertions: true`, `signDocuments: false`, `wantAuthnRequestsSigned: false`, `nameIdFormat: DEFAULT_NAME_ID_FORMAT` | ✅ verbatim |
| C95 | `spEntityId`, `acsUrl`, `nameIdFormat` are required with their own messages | ✅ `samlServiceProviderSchema` reused verbatim |
| C96 | `spEntityId` and `acsUrl` are trimmed before saving | ✅ verbatim |
| C97 | When unconfigured, the action is a plain `Enable SAML` button — no floating save bar | ✅ verbatim |
| C98 | When configured, the floating save bar appears on change and Cancel resets | ✅ |
| C99 | The attribute-mapping section only exists once the config exists | ✅ verbatim |
| C100 | With no mapper, a prompt offers to add `email`, `first_name`, `last_name` in one gesture | ✅ verbatim (`COMMON_PROFILE_MAPPERS`) |
| C101 | Attribute mapper form: name required, source required, name format required | ✅ `samlAttributeMapperSchema` reused verbatim |
| C102 | A custom source requires a non-empty `customKey` — `Attribute key is required` | ✅ verbatim, same conditional field |
| C103 | The custom key is prefixed with `attribute:` before being sent | ✅ `toCustomAttributeSource` |
| C104 | After a successful add, the form resets but keeps the chosen source and name format | ✅ verbatim |
| C105 | Deleting a mapper asks `Stop sending <name>?` and warns the application may break | ✅ verbatim |
| C106 | Loading renders `Loading the SAML configuration…` | ✅ skeleton instead |
| C107 | The three signature toggles keep the labels Enabled / Disabled | ✅ they stay `SwitchField` — real booleans, not two natures (FK-27) |
| C108 | The SAML tab is shown for every client, whatever its protocol | ✅ preserved. Reported as an observation: the tab lets SAML be enabled on an `openid-connect` client. |

## `feature/page-client-maintenance-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| C109 | The maintenance toggle applies immediately, carrying the current reason and strategy | ✅ verbatim |
| C110 | Reason and strategy are edited locally; `undefined` means “not edited, use the server value” | ✅ same override semantics |
| C111 | The save bar appears when either override is set; Cancel clears both | ✅ verbatim |
| C112 | Saving re-sends `enabled` unchanged along with the reason and the strategy | ✅ verbatim |
| C113 | An empty reason is sent as `undefined` | ✅ verbatim |
| C114 | Default strategy is `expire` when the server has none | ✅ verbatim |
| C115 | The strategy select offers `Expire naturally` and `Terminate immediately` | ✅ two `ChoiceCards` (FK-26) — expire and terminate are two natures, not on/off |
| C116 | Whitelist entries are added and removed one at a time, immediately | ✅ verbatim |
| C117 | Removing uses the entry id, resolved from a `user_id → entry.id` / `role_id → entry.id` map | ✅ verbatim |
| C118 | Realm-level whitelist entries are listed as inherited and cannot be removed here | ✅ verbatim, labelled `managed at the realm` (FK-32) |
| C119 | Inherited user and role labels fall back to the raw id when the entity is not in the realm list | ✅ verbatim |
| C120 | Users are searched by username with the email as sub-label; roles by name with the description | ✅ `EntityPicker` (FK-28) |
| C121 | Returns `null` while the client is not loaded | ✅ the page-level skeleton covers it |
| C122 | Reason description: `Message displayed to blocked users on the login page.` | ✅ verbatim |
| C123 | Whitelist intro: `Users and roles allowed to authenticate during maintenance. Realm-level entries are inherited automatically.` | ✅ verbatim |

## Summary

123 rules inventoried.

- 111 carried over verbatim or in the kit's equivalent form.
- 6 carried with a documented change: C20, C28, C50, C60, C78, C85.
- 6 deliberately dropped: C4, C6, C14, C15, C16, C18.

## UI ↔ domain divergences found

Findings, not fixes. Each one is a place where the current screens say
something the domain does not.

| # | Divergence | Evidence |
|---|---|---|
| D1 | **`ScopeType` has three values, the UI knew two.** `page-client-scopes.tsx` and `assigned-scopes-table.tsx` both carry `scope.default_scope_type === 'DEFAULT' ? 'default' : 'optional'` under the comment *“This is a simplified approach”*. A scope stored as `NONE` was displayed as *Optional*, and its dropdown offered “Change to Default” — which unassigns an `optional` mapping that does not exist. | `Schemas.ScopeType = "NONE" \| "OPTIONAL" \| "DEFAULT"` in `front/src/api/api.client.ts`. Handled in the new view: `none` is its own state, the type switch and the remove button are inert for it (rule C78). |
| D2 | **The effective-scopes preview computes nothing.** It mirrors the assigned scopes into state through a `useEffect` and captions them *“Effective Scopes”*, plus a literal `Claims: Standard scope claims (implementation-specific)` line. The domain distinguishes default (always issued) from optional (issued on request); the preview did not. | `front/src/pages/client/ui/components/effective-scopes-preview.tsx`. See rule C85. |
| D3 | **`client.secret` can never be read from the client payload.** Any code reading its *value* gets `"***"`; only its presence carries information. | `Masked_String` in `front/src/api/api.client.ts`, produced by `Masked<String>` in `libs/maskass/src/masked.rs` — “always serializes/logs as `"***"`”. `layout/client-layout.tsx` correctly tests presence only (rule C23). |
| D4 | **The SAML tab is offered on every client, whatever its `protocol`.** An administrator can register a SAML service provider on an `openid-connect` client: `saml-config` is a separate resource keyed by `client_id`, with no protocol check visible from the front. The rule is preserved (C108), but FK-29 now makes the protocol a creation-time and final decision, which makes this reachable state look accidental rather than intended. | `front/src/pages/client/layout/client-layout.tsx` lists the tab unconditionally; `PUT /realms/{realm_name}/clients/{client_id}/saml-config` takes `SetClientSamlConfigValidator`, which carries no protocol. |
| D5 | **`require_pkce` cannot be set at creation.** It exists on `Client` and on `UpdateClientValidator`, but not on `CreateClientValidator`. A public client is therefore always created without PKCE and has to be edited immediately after — the exact case whose own help text reads *“Strongly recommended: this client cannot keep a secret safe.”* (rule C40). | `Schemas.CreateClientValidator` vs. `Schemas.UpdateClientValidator` in `front/src/api/api.client.ts`. |
| D6 | **No endpoint rotates a client secret.** FK-34’s “Regenerate” button has nothing to call, so the credentials tab names the absence instead of offering a dead control. | The only route is `GET /realms/{realm_name}/clients/{client_id}/client-secret`. |
| D7 | **`ClientType` has a third value, `system`.** Neither console surfaces it; the displayed type is derived from `public_client`, so a `system` client reads as *confidential*. | `Schemas.ClientType = "confidential" \| "public" \| "system"`. |

**D4 and D5 read together** are the sharpest pair: the protocol is now chosen
once and for ever at creation, yet the creation payload cannot carry the one
setting a public client most needs, and the SAML tab stays reachable on a
client whose protocol says it will never speak SAML. Both belong to the
creation endpoint, not to the views.
