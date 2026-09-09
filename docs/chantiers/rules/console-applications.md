# Business rules inventory — `console/applications`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Source screens: `front/src/pages/console-applications/` — `page-console-applications.tsx`,
`types.ts`, `feature/` (4 files), `ui/page-applications-list.tsx`,
`ui/page-create-application-type.tsx`, `ui/page-create-application.tsx`,
`ui/application-detail-shell.tsx`, `ui/tabs/` (7 files).

Target: `front/src/next/pages/ciam/applications/`.

## The application-type mapping

An "application" is a `Client`. The five customer-facing types are a vocabulary
over four boolean fields of that client; nothing else in the codebase writes
this mapping down. Reproduced verbatim from
`front/src/pages/console-applications/feature/page-create-application-feature.tsx`
(`payloadFor`) and `types.ts` (`inferApplicationType`).

### What creation writes

Every type also sets `protocol: 'openid-connect'`, `enabled: true` and
`direct_access_grants_enabled: false`.

| Type | `client_type` | `public_client` | `service_account_enabled` | `oauth_device_code_grant_enabled` | Secret generated server-side |
|---|---|---|---|---|---|
| `native` — mobile or desktop | `public` | `false` | `false` | `false` | **yes** |
| `spa` — single-page app | `public` | `true` | `false` | `false` | no |
| `web` — server-rendered | `confidential` | `false` | `false` | `false` | yes |
| `m2m` — machine to machine | `confidential` | `false` | `true` | `false` | yes |
| `device` — device / CLI | `public` | `true` | `false` | `true` | no |

The last column is not a UI decision: `core/src/domain/client/services.rs:188`
reads `let secret = (!input.public_client).then(generate_random_string)`. The
secret follows `public_client`, never `client_type`. See the divergence section.

### What reading back infers

Order matters; the first match wins.

1. `service_account_enabled` → `m2m`
2. `oauth_device_code_grant_enabled` **and** no redirect URI → `device`
3. `client_type === 'public'` → `public_client ? 'spa' : 'native'`
4. otherwise → `web`

`client_type === 'system'` therefore reads back as `web`.

### What the type commands downstream

| Type | Authorization code flow | Callback URLs | Web origins | Require PKCE row | SAML tab |
|---|---|---|---|---|---|
| `native` | yes | required | shown | shown | shown |
| `spa` | yes | required | shown | shown | shown |
| `web` | yes | required | shown | shown | shown |
| `m2m` | no | hidden | hidden | hidden | hidden |
| `device` | no | hidden | hidden | hidden | hidden |

The original hides web origins at creation for `native` and `web` but shows
them in the settings tab for all three authorization-code types. Carried over
as-is.

## `ui/page-applications-list.tsx`

| # | Rule | Carried over |
|---|---|---|
| A1 | Title `Applications`, subtitle naming the four kinds of client | ✅ reworded to say what an application *does* (FK-22) |
| A2 | Six stat cards: total + one per type, each doubling as the type filter | ⚠ split: the five type counts become the filter chips (`SPA (3)`), the metrics band carries four figures (FK-12 caps it at four columns) |
| A3 | Metrics band content | ⚠ changed: Total, Active, Hold a secret, Missing a callback URL. `Hold a secret` reads `client.secret`, not `client_type` — see D1 |
| A4 | Status filter: All / Enabled / Disabled | ⚠ partially — `All` and `Disabled` survive as chips; `Enabled` is dropped: `ListingPage` filters are single-select, so status and type could no longer be combined, and `All` minus `Disabled` is the same set. Reported. |
| A5 | Type filter, single-select, `all` included | ✅ the filter chips |
| A6 | Search matches `name` or `client_id`, case-insensitive | ✅ `searchIn` |
| A7 | Sort: `Most recent` (`created_at` desc) or `Name (A→Z)` | ✅ every column is sortable in `DataView`; `Created` and `Application` carry the same two orders |
| A8 | Grid of cards, one per application | ✅ `DataView` list view by default, card view one click away (FK-13) |
| A9 | Card shows icon per type, `name || client_id`, `client_id` in mono | ✅ `Squircle`, title, mono subtitle |
| A10 | Card shows an `Off` badge when `!enabled` | ✅ `StatusDot` + `enabled`/`disabled` pill |
| A11 | Card footer: type short label, auth flow, relative created date | ✅ list columns Type / Sign-in flow / Created (`formatDate`, not a hand-rolled relative formatter) |
| A12 | Loading renders 6 skeleton cards | ✅ `DataView loading` |
| A13 | Two distinct empty states: no application at all vs. filtered out | ✅ `emptyLabel` / `emptyHint` + `ListingPage` filtered-out state |
| A14 | The empty state repeats the create button | ✅ `emptyAction` |
| A15 | Primary action `Create application` | ✅ opens the type picker |
| A16 | Relative date helper (`just now`, `5m ago`, …) | ❌ dropped — `@/next/shared/format-date` is the shared formatter and the column is a date, not a feed |
| A17 | Per-type colour tones (blue/emerald/amber/violet/rose) | ⚠ mapped onto the kit `PillTone` scale: native `info`, spa `success`, web `primary`, m2m `violet`, device `neutral`. No hard-coded colours (FK-01). |
| A18 | — (new) an application of an authorization-code type with no callback URL cannot complete a sign-in | ✅ counted as a metric and raised as a warn alert naming the applications |

## `ui/page-create-application-type.tsx` (step 1)

| # | Rule | Carried over |
|---|---|---|
| A19 | Type is chosen on a dedicated page before the form | ⚠ FK-29: it becomes a blocking `CreatePickerDialog` on the listing, and travels as `?type=` |
| A20 | Five cards: icon, label, description, short badge, auth flow | ✅ `Choice` options; the flow sentence is appended to the description |
| A21 | `Continue` is disabled until a type is picked | ✅ the dialog pre-selects `spa` and cannot be submitted empty |
| A22 | Step indicator `Step 1 of 2` | ❌ dropped with the two-page wizard |
| A23 | Footnote: *You can change the application type later, but some settings will reset.* | ❌ dropped — it is false. No screen, endpoint or field lets a type be changed. Replaced by the opposite statement on both the creation page and the settings tab. Reported. |
| A24 | `Cancel` returns to the listing | ✅ the dialog closes onto the listing |

## `ui/page-create-application.tsx` + `feature/page-create-application-feature.tsx` (step 2)

| # | Rule | Carried over |
|---|---|---|
| A25 | An unknown or missing `:type` sends the user back to step 1 | ✅ `<Navigate to={listUrl + '?create=1'} replace />` (FK-29) |
| A26 | `client_id` is auto-derived from the name by slugify (lowercase, NFD strip, non `[a-z0-9-_]` → `-`, trim `-`) | ✅ verbatim |
| A27 | Typing in the client ID field freezes the derivation | ✅ verbatim (`clientIdOverride`) |
| A28 | `client_id` must match `^[a-z0-9-_]+$` — *Only lowercase letters, numbers, hyphens and underscores.* | ✅ verbatim |
| A29 | Name is required | ✅ `createClientSchema` reused verbatim (*The name is required*) |
| A30 | Client ID is required | ✅ same schema (*The client ID is required*) |
| A31 | Callback URLs must match `^scheme://…` (custom mobile schemes accepted) | ✅ same regex, raised as a field error when a chip is refused |
| A32 | At least one callback URL is required for `native`, `spa`, `web` | ✅ verbatim |
| A33 | Web origins validated by `isWebOriginValue` (`+` sentinel accepted) | ✅ `@/lib/web-origin` reused verbatim |
| A34 | Web origins shown only for `spa`, always optional | ✅ verbatim |
| A35 | Per-type labels, hints and placeholders for the URL fields | ✅ carried and extended with the consequence of each field (FK-22) |
| A36 | `m2m` shows a banner explaining `client_credentials` and the generated secret | ✅ becomes a section; a symmetric one was added for `device` |
| A37 | Submit disabled while submitting | ✅ `canSubmit` drops the `SaveBar` while the request is in flight |
| A38 | Create the client, then POST each callback URL, then each web origin | ✅ verbatim, same order |
| A39 | A failed URL registration toasts `Could not register callback URL: <url>` and keeps the created client | ✅ verbatim, same for web origins |
| A40 | Success toast `Application created`, then navigate to the listing | ✅ verbatim |
| A41 | Failure toast carries the error message, or `Failed to create application` | ✅ verbatim |
| A42 | Empty URL rows are filtered out before submitting | ✅ n/a — `ChipInput` never holds an empty entry |
| A43 | Header repeats the type: icon, `Configure your <type>`, short badge, description, auth flow | ✅ a `Pill` with the type plus a *What this type gives you* section stating the flow and whether a secret is generated |
| A44 | `Change type` link back to step 1 | ✅ *change type* link to the picker (FK-29, same as the IAM `change protocol`) |

## `ui/application-detail-shell.tsx` + `feature/page-application-detail-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| A45 | Header: type icon, `name || client_id`, type badge, `Off` badge, `client_id` in mono | ✅ `Squircle`, title, mono `client_id`, type pill, enabled pill, maintenance pill |
| A46 | Back link `Back to applications` | ✅ |
| A47 | Tab order: Quickstart, Settings, Credentials, Connections, API Access, SAML, Addons, Login Experience, Maintenance | ⚠ preserved minus the three disabled ones: Quickstart, Settings, Credentials, API access, SAML, Maintenance |
| A48 | `Connections`, `Addons`, `Login Experience` render a `Coming soon` placeholder and cannot be selected | ❌ dropped — three tabs that cannot be opened and call nothing. Same call as the IAM made on the secret rotation button. Reported. |
| A49 | `Connections`, `SAML`, `Addons`, `Login Experience` are hidden for `m2m` and `device` | ✅ for the one that survives: SAML shows only for the authorization-code types |
| A50 | Active tab is held in `?tab=`, unknown or disabled value falls back to Quickstart | ⚠ FK-16: the tab is a path segment (`/applications/:id/settings`), unknown segment falls back to the first tab. `/applications/:id` redirects to `quickstart`. |
| A51 | Loading renders a skeleton | ✅ |
| A52 | — (new) an application that does not exist | ✅ *Application not found* panel, as in the IAM |

## `ui/tabs/quickstart-tab.tsx`

| # | Rule | Carried over |
|---|---|---|
| A53 | Seven endpoints derived from `window.apiUrl` + `/realms/<realm>` | ✅ same seven, each with a copy button |
| A54 | JWKS endpoint is `…/protocol/openid-connect/certs` | ❌ **corrected** to `…/protocol/openid-connect/jwks.json`. The server publishes the second one; the first 404s. Evidence and reasoning in D2. |
| A55 | Client ID shown with a copy affordance | ✅ its own section, saying the client ID is not a credential |
| A56 | Snippet branches on `m2m` (client credentials) vs everything else (authorization code) | ⚠ a third branch was added for `device`, using the real endpoint `…/protocol/openid-connect/auth/device` (`libs/ferriskey-api-authentication/src/router.rs:79`). The device archetype previously received an authorization-code snippet it cannot run. |
| A57 | The authorization-code snippet uses the first registered redirect URI, or a placeholder | ✅ verbatim |
| A58 | Snippet is copyable | ✅ |

## `ui/tabs/settings-tab.tsx`

| # | Rule | Carried over |
|---|---|---|
| A59 | Editable: `name`, `enabled`, `direct_access_grants_enabled`, `oauth_device_code_grant_enabled`, `require_pkce`, and the four token lifetimes | ✅ same set |
| A60 | `client_id` is **not** editable here (unlike the IAM settings tab) | ✅ shown read-only, with the reason (FK-22) |
| A61 | Redirect URIs and web origins sections render only for the authorization-code types | ✅ verbatim |
| A62 | `Require PKCE` renders only for the authorization-code types | ✅ verbatim |
| A63 | The PKCE description gains a *strongly recommended* sentence when the application cannot hold a secret | ✅ same condition, expressed through the type (`spa`, `device`) rather than through `client_type` |
| A64 | Grants section is shown for every type | ✅ |
| A65 | Web origins description mentions the `+` sentinel and that regex patterns are skipped | ✅ verbatim, plus the IAM's cache-propagation sentence |
| A66 | Redirect URIs and web origins are saved on add/remove, not on Save | ✅ `ChipInput`, one call per entry, toast per outcome |
| A67 | Token lifetimes: four numeric fields, empty = inherit the realm default | ✅ product `DurationInput` with `nullable` |
| A68 | Save bar appears only when the form is dirty, with a `Discard` | ✅ `SaveBar` with a dirty count |
| A69 | Danger zone with a confirmation quoting `name || client_id` | ✅ `DangerZone` with retyped-name confirmation |
| A70 | Deleting toasts `Application deleted` and returns to the listing | ✅ verbatim |
| A71 | Section order: General, Redirect URIs, Web origins, Grants, Security, Token lifetimes, Danger zone | ✅ preserved; the two URL sections are merged into one *Where users come back*, and Security folds into *Sign-in methods* |
| A72 | Field help texts (`Disabled applications cannot start a sign-in flow.`, `Allow exchanging a username/password…`, …) | ✅ carried and rewritten to state the consequence (FK-22) — e.g. the password exchange row now says MFA and social sign-in never run |

## `ui/tabs/credentials-tab.tsx`

| # | Rule | Carried over |
|---|---|---|
| A73 | Confidential applications get the secret section; public ones get an explanation instead | ⚠ the branch now reads `client.secret` rather than `client_type === 'confidential'` (FK-34: the presence is the meaningful part, and `native` is `client_type: 'public'` yet holds a secret — D1) |
| A74 | The secret is fetched only once `Reveal` is pressed, and never cached | ✅ IAM `ClientCredentialsTabFeature` reused verbatim (`useGetClientSecret`, `gcTime: 0`, no refetch) |
| A75 | Revealing is announced as a recorded security event | ✅ verbatim |
| A76 | HTTP 403 shows *You need the manage-clients permission…* | ✅ verbatim |
| A77 | Any other failure shows *The secret could not be revealed. Please try again.* | ✅ verbatim |
| A78 | Copy button disabled until the secret is revealed | ✅ verbatim |
| A79 | A disabled `Rotate` button with a `Soon` badge | ❌ dropped — no rotation endpoint exists. The IAM already replaced it with the sentence *Rotating the secret is not exposed by the administration API yet.* (FK-34) |
| A80 | Client ID copy row | ✅ in both branches |

## `ui/tabs/api-access-tab.tsx`

| # | Rule | Carried over |
|---|---|---|
| A81 | Assign realm client scopes to the application as `default` or `optional` | ✅ IAM `ClientScopesTabFeature` reused verbatim |
| A82 | Already-assigned scopes are excluded from the picker | ✅ verbatim |
| A83 | Switching a scope between default and optional = unassign then assign | ✅ verbatim |
| A84 | Scopes are grouped, default first, with *Always granted* / *Granted only when requested* | ✅ the IAM tab groups them by pill tone and a segmented view |
| A85 | Multi-select composer assigning several scopes in one go, sequentially | ❌ not carried — the IAM dialog assigns one scope at a time. Reported. |
| A86 | `Token preview` listing the default scope names | ❌ not carried — the IAM tab ships a full evaluation panel (pick a user, pick optional scopes, evaluate) which supersedes it |
| A87 | Empty state *No scopes assigned* | ✅ the IAM `EmptyState` |

## `ui/tabs/saml-tab.tsx`

| # | Rule | Carried over |
|---|---|---|
| A88 | Entity ID and ACS URL are required before saving | ✅ IAM `ClientSamlTabFeature` reused verbatim (`samlServiceProviderSchema`) |
| A89 | Not-yet-configured banner, and the submit reads `Enable SAML` instead of `Save changes` | ✅ verbatim |
| A90 | Name ID format select with per-option description | ✅ verbatim |
| A91 | Three signature switches, `sign_assertions` on by default | ✅ verbatim |
| A92 | Attribute mappers: add, delete, and a one-click *email, first_name, last_name* | ✅ verbatim |
| A93 | A custom source requires an attribute key before the mapper can be added | ✅ verbatim |
| A94 | The SAML tab is offered on applications created with `protocol: 'openid-connect'` | ✅ behaviour kept — see D3 |

## `ui/tabs/coming-soon-tab.tsx` and `ui/tabs/primitives.tsx`

| # | Rule | Carried over |
|---|---|---|
| A95 | `ComingSoonTab` with icon, title, description, bullet points | ❌ dropped with A48 |
| A96 | `Section`, `Field`, `CopyRow` local primitives | ❌ replaced by the kit `Section` / `FieldRow` and a local copy row |

## Maintenance tab

| # | Rule | Carried over |
|---|---|---|
| A97 | The maintenance tab renders the admin console's `PageClientMaintenanceFeature` verbatim | ✅ now the IAM `/next` `ClientMaintenanceTabFeature`, which is the migrated version of that same screen |

## Divergences found between the UI and the domain

**D1 — a `native` application is a confidential client in everything but its label.**
Creation writes `client_type: 'public'` with `public_client: false`
(`feature/page-create-application-feature.tsx:31`). The server keys the secret
off `public_client` alone — `core/src/domain/client/services.rs:188`,
`let secret = (!input.public_client).then(generate_random_string)` — and the
token endpoint requires that secret for the same reason
(`core/src/domain/authentication/services.rs:380`,
`if !client.public_client && !client_secret_matches(…)`). So a mobile or
desktop application created from this console is issued a secret it cannot
keep, and its token exchange will be refused unless it ships that secret. The
credentials tab of the old console hid the secret behind
`client_type === 'confidential'`, so the user never saw it either. Not fixed:
the mapping is reproduced verbatim, and the credentials tab now branches on
`client.secret` so at least the secret that exists is visible.

**D2 — the JWKS URL published by the quickstart is wrong.**
`ui/tabs/quickstart-tab.tsx:24` advertises
`…/protocol/openid-connect/certs`. The discovery handler
(`libs/ferriskey-api-authentication/src/handlers/openid_configuration.rs:75`)
publishes `jwks_uri: …/protocol/openid-connect/jwks.json`. This is the one
divergence corrected in place rather than only reported: it is a URL handed to
a customer to paste into their code, and carrying it over unchanged would ship
a copy-paste 404.

**D3 — the SAML tab ignores `client.protocol`.**
Applications are always created with `protocol: 'openid-connect'`, yet the SAML
tab is offered on every browser-facing application and writes a SAML service
provider configuration for it. The IAM console gates the same tab on
`client.protocol === 'saml'`. Either the protocol field is not what decides
whether SAML answers for a client, or the customer console can produce a client
that is SAML-configured while declaring itself OIDC. Behaviour kept as the
current console has it; reported, not fixed.

**D4 — the type-picker footnote promised something that does not exist.**
See A23. Nothing in the API changes a client's `client_type`,
`public_client` or `service_account_enabled` after creation — `PATCH
/clients/{id}` accepts none of them.

**D5 — `require_pkce` cannot be set at creation.**
`CreateClientValidator` has no `require_pkce` field, so every application —
including SPAs and native apps, which have nothing else protecting the code
exchange — is created with PKCE optional and must be hardened afterwards from
the settings tab. Reported, not worked around.
