# Business rules inventory — `identity-providers`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Source screens: `front/src/pages/identity-providers/ui/page-overview.tsx`,
`ui/page-detail.tsx`, `ui/page-create.tsx`, their features, their components
and `layouts/providers-layout.tsx`. Those files are **read-only**: the domain
is also mounted by the CIAM product
(`pages/console-authentication/page-console-authentication.tsx` imports
`PageIdentityProviders` whole).

## `ui/page-overview.tsx` + `feature/page-overview-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| P1 | Not loading and no provider → a dedicated empty state replaces the statistics and the list | ✅ `ListingPage` `emptyLabel` / `emptyHint` / `emptyAction` (FK-13) |
| P2 | Four statistics: total, enabled, disabled, number of **distinct** `provider_id` | ✅ `MetricsBand`, same four |
| P3 | The enabled statistic shows `X% active` when enabled > 0 **and** total > 0, otherwise the literal `No enabled providers` | ✅ `hint`, wording kept |
| P4 | The row avatar is coloured by provider type (`oidc`, `oauth2`, `saml`, `ldap`, fallback violet) | ✅ as a `Pill` tone on the type column, and the real brand logo as avatar |
| P5 | Row title is `display_name \|\| alias`; the alias is repeated underneath in mono | ✅ title column + dedicated mono `Alias` column |
| P6 | A mono badge carries the raw `provider_id` | ✅ `Pill mono` |
| P7 | A badge carries `enabled` / `disabled` | ✅ `Pill` + `StatusDot` (FK-30 — shape *and* colour) |
| P8 | Search covers `display_name` and `alias` only | ✅ `searchIn` |
| P9 | Clicking a row opens the provider detail | ✅ `getHref` |
| P10 | Per-row delete opens `ConfirmDeleteAlert`, titled `Delete provider?`, quoting the display name | ✅ unchanged, in a trailing column |
| P11 | Delete raises a success or an error toast naming the provider | ✅ unchanged |
| P12 | The empty state offers four popular providers (google, discord, github, microsoft) that pre-select the template through `?provider=<id>` | ✅ kept in `emptyAction`, `?provider=` plus `?protocol=` (FK-29) |
| P13 | `id` of a row is the **alias** — it is the API path parameter, not `internal_id` | ✅ `getKey` and the detail route use the alias |
| P14 | `display_name` falls back to the alias in the list mapping | ✅ |
| P15 | The header carries the title, the description and the `Add Provider` primary action | ✅ `ListingPage` header |
| P16 | Bulk delete (`handleDeleteSelected`) | ❌ **dropped** — the prop was passed but never wired to any control in the current view; nothing could invoke it. Reported, not reproduced. |
| P17 | `updated_at` is mapped to a hard-coded `null`, and `columns/list-provider.column.tsx` renders it as `—` | ❌ **dropped** — the column file is dead code (the view uses `OverviewList`, not `DataView`) and the API never returns a timestamp. See *UI ↔ domain divergences*. |

## `ui/page-detail.tsx` + `feature/page-detail-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| D1 | Loading renders a skeleton, not an empty state | ✅ |
| D2 | No provider → `Provider not found.` with the back link kept | ✅ wording expanded, back link kept |
| D3 | Provider type label map `oidc → OIDC`, `oauth2 → OAuth2`, `saml → SAML`, `ldap → LDAP`, falling back to the raw `provider_id` | ✅ verbatim |
| D4 | Header title is `display_name ?? alias`, subtitle repeats the alias | ✅ title + mono alias `Pill` |
| D5 | Type badge and enabled badge sit in the header | ✅ |
| D6 | The alias field is rendered **disabled** | ✅ and now says why (FK-23): it is sealed into the Redirect URI already declared at the provider |
| D7 | Only `display_name` and `enabled` are editable | ✅ |
| D8 | The enabled control shows its current state as a label | ✅ `ChoiceCards` (FK-26) — two named states with their consequence |
| D9 | The configuration section lists **every** key of `config` | ✅ |
| D10 | A configuration key is humanised: `token_url` → `Token Url` | ✅ |
| D11 | A value whose key contains `secret` or `credential` is displayed masked | ✅ `••••••••`, with the reason: the API never returns it in clear |
| D12 | Every configuration input is disabled — configuration is read-only on this screen | ✅ rendered as a definition list, not as dead inputs (FK-11) |
| D13 | Empty configuration → `No configuration settings.` | ✅ FK-32, wording kept in substance |
| D14 | Metadata section: `internal_id` and `provider_id` always | ✅ `internal_id` moves to the header (FK-11), `provider_id` stays in the section |
| D15 | `first_broker_login_flow_alias` row only when the value is present | ⚠️ **deliberately changed** — always rendered, and named `Not set` when absent (FK-32: an absence is named, never hidden) |
| D16 | Danger zone, confirmation quoting `display_name ?? alias` | ✅ product `DangerZone`, unchanged |
| D17 | The save bar appears only when the form is dirty; Cancel resets it | ✅ `FloatingActionBar` with a dirty count |
| D18 | The update request sends `display_name` and `enabled` only | ✅ |
| D19 | Toasts on update success, delete success, delete failure | ✅ |

## `ui/page-create.tsx` + `feature/page-create-feature.tsx` + components

| # | Rule | Carried over |
|---|---|---|
| C1 | Three steps, in this order: *Select Provider* → *Configure* → *Review* | ✅ |
| C2 | `?provider=<templateId>` pre-selects the template and opens the wizard at step 2 | ✅ |
| C3 | Changing the `provider` parameter resets the selection and the step | ✅ derived at render from the URL, no `useEffect` |
| C4 | Selecting a template auto-advances to step 2 | ✅ |
| C5 | Step 2 → 3 only when the form validates | ✅ |
| C6 | `Next` is disabled without a template (step 1) or on an invalid form (step 2) | ✅ |
| C7 | The template pre-fills display name, authorization URL, token URL, userinfo URL and the default scopes; credentials stay blank | ✅ |
| C8 | Display name is required, max 50 | ✅ (`createProviderSchema` requires it; the 50-char cap is kept) |
| C9 | Client ID required, Client Secret required | ✅ |
| C10 | URLs must be valid URLs | ✅ |
| C11 | The client secret is masked, with a show/hide toggle | ✅ `InputGroup` + eye toggle |
| C12 | Advanced settings are collapsed, except for the custom template where they are open | ⚠️ **deliberately changed** — endpoints are a plain `Section`, always visible. They are required by the server for every template (see *UI ↔ domain divergences*), so hiding them behind a disclosure hid a mandatory field. |
| C13 | The custom template marks Authorization URL and Token URL as required | ✅ required for every template now, and the section says why |
| C14 | A non-custom template warns that the URLs are pre-configured | ✅ in the section description |
| C15 | The default scopes are shown as badges | ✅ chips in the form and the raw list in the help rail |
| C16 | The gallery searches on `displayName` + `description` | ✅ |
| C17 | The gallery groups by category in the order social, enterprise, developer, custom | ✅ |
| C18 | The custom card is always appended to the custom group, even when the search filters it out | ✅ |
| C19 | `No providers found matching '<query>'` when the search returns nothing | ✅ |
| C20 | Help panel: logo, protocol in caps, documentation link, Redirect URI with a copy button, setup steps, an info alert, the default scopes | ✅ the rail keeps all seven blocks |
| C21 | Setup steps differ for the custom template | ✅ both lists kept verbatim |
| C22 | Callback URL = `${window.apiUrl}/realms/${realm}/broker/<alias>/endpoint` | ✅ — built from the **alias**, see C26 |
| C23 | The review shows the summary, the credentials (secret masked), the OAuth configuration and the callback URL | ✅ |
| C24 | On create: `enabled: true`, `store_token: false`, `add_read_token_role_on_create: false`, `trust_email: true`, `link_only: false` | ✅ verbatim |
| C25 | `display_name` falls back to `template.displayName` | ✅ |
| C26 | `alias` **and** `provider_id` are both set to `template.name` | ⚠️ **deliberately changed** — the alias is now typed by the administrator (slugified) and only `provider_id` comes from the template. See *UI ↔ domain divergences*: the current behaviour makes a second provider of the same brand impossible, and the alias is the realm-unique key. |
| C27 | `config`: `client_id`, `client_secret`, `authorization_url` (falling back to the template), `token_url` (idem), `userinfo_url` only when non-empty, `scopes` from the form or the template defaults | ✅ with `scopes` **always** emitted — see *UI ↔ domain divergences* |
| C28 | Success → toast + navigation back to the listing | ✅ (via `onSuccess`, not a `useEffect`) |
| C29 | Cancel, and Back on step 1, return to the listing | ✅ |
| C30 | The create button is disabled while the mutation is pending | ✅ |

## FK rules applied on top

| Rule | Where |
|---|---|
| FK-13 | `ListingPage` distinguishes "no provider yet" from "the filter hides them" |
| FK-14 | Misconfigured providers are raised in the listing alert band with a verb |
| FK-23 | The alias is disabled **and** justified, on the detail and in the wizard |
| FK-26 | `enabled` becomes two named cards instead of a bare switch |
| FK-29 | The protocol is chosen from the listing through `CreatePickerDialog`, travels as `?protocol=`, and `/create` without it redirects to the listing |
| FK-30 | Configuration state is a pill with an icon *and* a colour |
| FK-32 | `Not set`, `No scope declared`, `No configuration recorded` — no blank cell |
| FK-40 | The picker offers `oidc` and `oauth2` only; `saml` and `ldap` are shown inert with the reason |

## UI ↔ domain divergences — reported, not fixed

**1. `saml` and `ldap` are offered by the front and implemented nowhere.**
`front/src/pages/identity-providers/schemas/create-provider.schema.ts`
declares `providerTypeSchema = z.enum(['oidc', 'oauth2', 'saml', 'ldap'])` and
both the listing and the detail carry `SAML` / `LDAP` labels and colours. The
broker only speaks OAuth2/OIDC: `IdentityProviderConfig` converts into
`OAuthProviderConfig` and nothing else
(`libs/ferriskey-abyss/src/identity_provider/broker/value_objects.rs:71`,
`core/src/domain/abyss/broker_services.rs:497`). There is no SAML or LDAP
identity-provider broker in the tree (`core/src/domain/saml` is FerrisKey
acting as a SAML **IdP**, not as a service provider). Not fixed: the labels are
kept where they are only descriptive, and the creation picker shows the two
unsupported types inert with the reason (FK-40) rather than silently removing
them.

**2. `scopes` is required by the broker but optional in the creation form.**
`OAuthProviderConfig.scopes` is `Vec<String>` with no `serde` default
(`broker/value_objects.rs:29`); a config without the key fails to deserialize
and `CoreError::InvalidProviderConfiguration` is raised at login time, not at
creation time. The current form only writes `scopes` when the form value or
the template default is non-empty — so a custom provider created with no scope
produces a provider that cannot broker a login. The new form always emits the
key. Not fixed server-side: adding a `#[serde(default)]` is a domain change.

**3. `authorization_url` and `token_url` are required by the broker and optional in the form.**
Same file: both are `String`, not `Option<String>`. The current form accepts
them empty and falls back to the template — which is the empty string for the
custom template. The new form requires them.

**4. The alias is the realm-unique key, and the form does not let anyone set it.**
`IdentityProvider.alias` is documented as *"Unique alias within the realm"*
(`libs/ferriskey-abyss/src/identity_provider/entities.rs:54`) and it is the
path parameter of every single-provider route
(`/realms/{realm_name}/identity-providers/{alias}`). The current wizard writes
`alias = provider_id = template.name`, so a realm can hold exactly one Google
provider, and no administrator can name one. Fixed in the new wizard (C26)
because it is the difference between a usable and an unusable screen; flagged
here because it changes what the create call sends.

**5. The API never returns `created_at` / `updated_at` for a provider.**
The domain entity carries both (`entities.rs:86-90`) but
`IdentityProviderResponse`
(`libs/ferriskey-api-abyss/src/identity_provider/dto.rs:74`) drops them. The
current listing therefore maps `updated_at: null` and its `Last Updated`
column can only ever print `—`. The new listing has no such column. Not fixed:
adding the fields to the DTO regenerates `api.client.ts`, which this chantier
does not touch.

**6. The client secret is already masked by the server.**
`IdentityProviderConfig.client_secret` is `Masked<String>` and serializes to
the literal `"***"` (`libs/maskass/src/masked.rs:96`). The current detail masks
it a second time into `••••••••`, which reads as if the console were hiding a
value it holds. The masking is kept (D11) but the description now says the
value never leaves the server.

**7. There is no health, no linked-account count and no `status` on a provider.**
The prototype's `status`, `statusDetail`, `linkedAccounts` and `trustEmail`
columns come from its mock model. `trust_email` exists; the other three do not.
The listing derives a **configuration state** instead — healthy / degraded /
error, computed from the keys `OAuthProviderConfig` requires — which is the
only provider state the domain can actually answer. Linked accounts exist per
user (`/users/{user_id}/identity-provider-links`) with no per-provider
aggregate, so no count is shown.
