# Business rules inventory — `user-federation`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Source screens: `front/src/pages/user-federation/` — `ui/page-overview.tsx`,
`ui/ldap-form-ui.tsx`, `ui/kerberos-form-ui.tsx`, `layout/user-federation-layout.tsx`,
plus the logic the features hold (`feature/page-overview-feature.tsx`,
`feature/page-create-ldap-feature.tsx`, `feature/page-detail-ldap-feature.tsx`,
`feature/page-create-kerberos-feature.tsx`).

## `ui/page-overview.tsx`

| # | Rule | Carried over |
|---|---|---|
| F1 | Two provider types are known to the listing, LDAP and Kerberos; the type is matched case-insensitively and an unknown type falls back to the LDAP presentation | ⚠️ partly — the type is shown verbatim as the domain returns it (`Ldap`, `Kerberos`, `ActiveDirectory`, custom), no fallback renaming. Renaming an unknown type into "LDAP" hides what the record actually is |
| F2 | Four statistics: total, one per type, active | ⚠️ replaced — `MetricsBand` shows Providers / Enabled / Scheduled sync / Never synced. The per-type counts became a `LDAP` filter chip; "never synced" is the number the domain lets us act on |
| F3 | Clicking a statistic card applies the matching filter; the "Active" card also resets the type filter | ✅ equivalent — filter chips of `ListingPage` (`Enabled`, `LDAP`, `Scheduled`, `Never synced`) |
| F4 | Status filter All / Active / Inactive | ✅ `Enabled` chip + `All` |
| F5 | Search covers name, type and connection string | ✅ `searchIn` covers name, `provider_type`, endpoint |
| F6 | Sort by "Most recent" (never-synced last) or "Name (A→Z)" | ✅ column sorting of `DataView` on Provider, Type, Sync mode, Schedule, Last sync, Status |
| F7 | Status is derived from `enabled` only — `syncing` is declared in the type and never produced | ✅ enabled / disabled only, the dead third state is dropped |
| F8 | "Last sync" reads `updated_at` and is rendered as a relative age | ❌ deliberately dropped: `updated_at` is the row's last write, not a synchronisation. The column now reads `last_sync_at`, which is the field the domain fills, and prints `never synced` when absent (FK-32) |
| F9 | The imported-account count is displayed and hardcoded to `0` — the API returns no such number | ❌ deliberately dropped: a constant `0` on every row states something false. No endpoint returns the count (`ProviderResponse` has no user count) |
| F10 | The connection string reads `config.connectionUrl ?? config.kdcServer ?? 'Unknown'` | ⚠️ corrected — those flat keys are never written by the console (the payload nests them under `config.connection`), so the column read `Unknown` for every row. The endpoint is now rebuilt from `config.connection.server_url` / `port` / `use_tls` |
| F11 | Priority integer mapped to a label: 0 Primary, 10 Secondary, 20 Development, anything else Custom | ✅ same mapping, the residual label is `Legacy` (the four labels the form offers) |
| F12 | Empty state distinguishes "no provider at all" (offers a creation) from "no match for the filters" | ✅ `ListingPage` handles both (FK-13) |
| F13 | A row opens the provider; a pencil button does the same; a bin button deletes | ⚠️ relocated — the whole row opens the provider. Deletion moved to the provider's Danger Zone, as in the `role` pilot; the kit's `DataView` links the row itself, and a button nested in that link is a trap |
| F14 | Deletion is confirmed by a blocking dialog quoting the provider name, then a success/failure toast | ✅ `DangerZone` + `ConfirmDeleteAlert`, confirmation quotes the name |
| F15 | The header action always creates an LDAP provider | ❌ replaced by FK-29: the type is chosen in `CreatePickerDialog` and travels in the URL (`/create?kind=Ldap`) |
| F16 | Loading renders skeletons rather than an empty state | ✅ `DataView loading` |

## `ui/ldap-form-ui.tsx` (create and edit)

| # | Rule | Carried over |
|---|---|---|
| F17 | Validation (`schemas/ldap-provider.schema.ts`): name required, connection URL required, base DN required, bind DN optional, bind password optional, user search filter free, sync interval ≥ 60, TLS boolean, priority among four labels | ✅ the same zod schema is reused verbatim by both new features |
| F18 | The sync interval is expressed in seconds in the form and floored to minutes for the API | ✅ same conversion, with `components/ui/duration-input.tsx` on the input side (FK-33) |
| F19 | The type selector is hidden in edit mode | ✅ the type is a disabled field with the reason written next to it (FK-23) |
| F20 | In create mode the Kerberos card is present but disabled, badged "Coming Soon" | ✅ Kerberos is offered in the picker with a `disabledReason` naming the real cause: the server stores it but neither tests nor synchronises it (FK-39, FK-40) |
| F21 | The bind password is a `password` field | ✅ `InputGroup` with a show/hide button |
| F22 | In edit mode a help text says "Leave empty to keep the existing password" | ⚠️ corrected — see divergence D5. Empty now means *unchanged* only while the configuration is otherwise untouched; when a connection field changes, the credential must be typed again, and the reason is written under the field (FK-34, FK-39) |
| F23 | Edit mode exposes "Test Connection" and "Sync Users"; each is disabled while either runs; labels become "Testing…" / "Syncing…" | ✅ moved to the page header as the prototype does, same disabled logic, plus a `title` explaining the refusal when the type or the disabled state forbids the action |
| F24 | A successful connection test toasts `result.message`; a failure toasts it as an error | ✅ plus a persistent result strip carrying the `latency_ms` the API returns in `details`, formatted per FK-33 |
| F25 | A successful test fires the confetti | ✅ fired from the handler, no `useEffect` |
| F26 | A sync toasts the non-zero counters joined by commas, falling back to `Processed N users` | ✅ verbatim |
| F27 | The save bar shows when the form is valid (create) or when it is valid *and* dirty (edit) | ✅ create: `canSubmit`; detail: dirty count, and the bar states why saving is blocked when it is |
| F28 | Create and edit differ in title, description and button label | ✅ two distinct pages |
| F29 | Success navigates back to the listing | ✅ on create and on delete. Saving a detail stays on the page and refreshes it — the pilot behaves that way and the user usually keeps working on the provider |

## Feature-level rules (payload construction)

| # | Rule | Carried over |
|---|---|---|
| F30 | The connection URL is parsed with `new URL`, prefixing `ldap://` when the scheme is missing | ✅ same, in `parseLdapEndpoint` |
| F31 | The port is taken from the URL, else 636 with TLS and 389 without | ✅ verbatim |
| F32 | `server_url` receives the hostname only | ✅ verbatim |
| F33 | An unparseable URL throws and is caught into a generic error toast after the click | ⚠️ corrected — the parse failure is now a field-level message and the submit is inert (FK-39) |
| F34 | The bind password is base64-encoded (`btoa`) before being sent, and base64-decoded (`atob`) when read back, `********` being treated as empty | ⚠️ the encoding is kept (the backend base64-decodes it, `core/src/infrastructure/abyss/federation/ldap.rs:73-91`). The decoding on read is dropped: the API masks the value, so there is nothing to decode and nothing to prefill (FK-34) |
| F35 | `sync_mode` is hardcoded to `Import` and `sync_enabled` to `true` | ❌ replaced: both are chosen in the form. The domain has three modes and the create endpoint takes them (D1) |
| F36 | `use_starttls: false`, `connection_timeout_seconds: 30` and the attribute map `uid`/`mail`/`givenName`/`sn` are hardcoded on every write | ✅ same defaults, but an existing value is preserved instead of being overwritten — the update replaces `config` wholesale server-side |
| F37 | Priority label → integer: Primary 0, Secondary 10, anything else 20 | ⚠️ corrected: `Legacy` now writes 30. The old mapping sent 20 for both `Development` and `Legacy`, so choosing `Legacy` silently reopened as `Development` |
| F38 | The detail page prefills the sync interval with `(sync_interval_minutes ?? 60) * 60` seconds | ✅ verbatim |
| F39 | The detail page shows "Loading provider…" then "Provider not found" | ✅ skeleton, then a named not-found panel |
| F40 | An update sends every field of the provider | ✅ and it always sends the three `sync_*` fields together — the handler defaults the missing ones (`sync_enabled=false`, `sync_mode=LinkOnly`) instead of preserving them (D4). `config` is only sent when a configuration field changed |

## `ui/kerberos-form-ui.tsx` and its feature

| # | Rule | Carried over |
|---|---|---|
| F41 | The Kerberos creation page redirects to the LDAP creation page on mount, so the form is unreachable | ❌ dropped, and not replaced by a working form — see F20 and D2 |
| F42 | Its submit handler logs to the console and toasts success without calling any API | ❌ dropped: it never created anything |
| F43 | Its schema declares `kerberosRealm` and `kdcServer` required, `adminServer` optional, `allowPasswordAuth` boolean | ❌ dropped with the form. Nothing in the domain reads those keys |

## `layout/user-federation-layout.tsx`

| # | Rule | Carried over |
|---|---|---|
| F44 | Page header: `Database` icon, title "User Federation", description "Manage and configure external user storage providers" | ✅ title kept, description rewritten to say what the resource is |
| F45 | A single tab, "Providers list", driven by a `useState` that nothing reads | ❌ dropped: one tab commanding nothing is chrome without a function |

## Divergences UI ↔ domain — reported, not fixed

**D1 — The synchronisation mode was never offered, and the prototype's
`readonly` / `writable` exists nowhere.**
`core/src/domain/abyss/federation/entities.rs:29-57` declares
`enum SyncMode { Import, Force, LinkOnly }`, serialised as those exact
PascalCase strings (no serde rename; `Display`/`FromStr` agree, and a unit test
at `entities.rs:103-123` pins them). The current console never shows the mode:
both `page-create-ldap-feature.tsx:63` and `page-detail-ldap-feature.tsx:195`
hardcode `sync_mode: 'Import'`. `readonly` / `writable` appear in neither the
domain nor the product console — they were a kit invention, already corrected in
the prototype. The new form offers the three real modes.

**D2 — `FederationType` has four variants; the console can configure one.**
`entities.rs:10-27`: `Ldap`, `Kerberos`, `ActiveDirectory`, `Custom(String)`.
Only `Ldap` and `ActiveDirectory` are functional — `services.rs:243-252`
(test connection) and `services.rs:272-279` (sync) reject everything else with
`"Provider type not supported for connection testing"` and
`CoreError::Configuration("Provider type does not support sync")`. Kerberos is
accepted at creation (the three parse sites,
`handlers/create_provider.rs:47-52`, `handlers/update_provider.rs:55-60`,
`repository.rs:42-47`, map any unknown string to `Custom` and never fail) but is
inert once stored. The prototype's premise — "four variants, two have a form" —
is half right: the two the *server* can actually run are `Ldap` and
`ActiveDirectory`, and `ActiveDirectory` is the one the console never offers,
although it goes through the same LDAP code path. Not fixed here: adding an
Active Directory creation flow is a feature, not a style migration.

**D3 — The 60-second minimum is a front-end invention.**
`front/src/pages/user-federation/schemas/ldap-provider.schema.ts:13` carries
`syncInterval: z.number().min(60)`. There is no interval validation anywhere in
Rust: the DTO field is `sync_interval_minutes: Option<i32>`
(`libs/ferriskey-api-abyss/src/federation/dto.rs:9-20`), the column is a plain
nullable `INTEGER` (`core/migrations/20251128072715_create-user-federation.up.sql:11`),
and `core/src/domain/abyss/federation/services.rs:101-102` carries the pending
`// TODO: Validate config based on provider type`. Zero and negatives are
accepted by the API. The rule is kept in the view because it is the only place
it exists, and it is now announced before the click rather than after (FK-39).

**D4 — A partial update silently resets the synchronisation.**
`handlers/update_provider.rs:62-81` rebuilds the whole sync block as soon as one
of the three sync fields is present, defaulting the others:
`sync_enabled.unwrap_or(false)` and `sync_mode.unwrap_or("LinkOnly")`. The
repository then overwrites the three columns (`repository.rs:194-198`). Sending
only a new interval therefore disables the synchronisation and switches the mode
to `LinkOnly`. The new detail feature always sends the three fields together;
the server-side default is left as it is.

**D5 — Saving the current edit page destroys the bind credential.**
`dto.rs:60-85` masks `bind_password_encrypted` to `********` on every read, and
`repository.rs:187-189` replaces `config` wholesale on update. The current edit
page reads that guard correctly (`page-detail-ldap-feature.tsx:78-88` turns
`********` into an empty field) but its write path does not:
`page-detail-ldap-feature.tsx:184-186` falls back to
`config?.bind?.bind_password_encrypted`, i.e. it writes the literal `********`
back into the stored configuration whenever the administrator leaves the field
empty. Nothing server-side preserves the previous secret. The new detail page
therefore refuses to save a configuration change while the credential has not
been retyped, and says why under the field. The server-side asymmetry — a
wholesale `config` replace against a masked read — is left untouched.

**D6 — The "encryption" is base64.**
`ldap.rs:73-91` base64-decodes the stored value and falls back to plaintext,
logging `"Using placeholder password decryption - implement proper KMS
integration"`; the field itself is commented `// TODO: Decrypt this`
(`ldap.rs:39-44`). The console does the matching `btoa`. Anyone with read access
to the database recovers the credential. The new screens keep the same encoding
— changing it would make every stored provider unreadable — and describe the
field as "stored encrypted and never returned", which is what the API boundary
does.

**D7 — A synchronisation started by hand ignores the configured mode.**
`handlers/sync_users.rs:37` hardcodes `SyncMode::Import` with the comment
`"Default to Import mode for safety (Force would disable missing users)"`, and
the endpoint takes no body. The configured mode only ever applies to a scheduled
run. The field description says so instead of implying the button honours it.

**D8 — The sync response drops what the domain measured.**
`SyncResult` (`value_objects.rs:41-55`) carries `duration_ms` and
`errors: Vec<SyncError>` (`username`, `external_id`, `error`), but
`SyncUsersResponse` (`dto.rs:153-186`) keeps only the five counters and the two
timestamps. The console therefore cannot show which accounts failed — the
prototype's "failed accounts" list has no data behind it. The duration is
recomputed from `completed_at - started_at`.

**D9 — The migration default disagrees with the enum.**
`create-user-federation.up.sql:10` sets `sync_mode VARCHAR(20) NOT NULL DEFAULT
'import'` in lowercase, while `Deserialize`/`FromStr` only accept `Import`. A
row created outside the API path fails to deserialize. Nothing in the console
can produce that row; reported for completeness.

**D10 — `front/src/api/user-federation.api.ts` does not carry the invalidation
defect fixed on `role.api.ts` (commit `c14a7f61`).** Every mutation there builds
its invalidation key from the same generated helper the queries use —
`window.tanstackApi.get('/realms/{realm_name}/federation/providers').queryKey`
and its `/{id}` counterpart — so the keys match the queries `useGetUserFederations`
and `useGetUserFederation` register, and `useGetUserFederation` carries no
`staleTime`. Nothing to report on that front; the file is out of scope and was
not touched.

## Deliberate deviations from the prototype

- **No "Linked accounts" tab.** The prototype lists `mappingsForProvider(...)`.
  `user_federation_mappings` exists as a table and the service batch-loads it
  (`services.rs:389-392`), but no HTTP route exposes it — the seven federation
  routes are the CRUD plus `test-connection` and `sync-users`
  (`libs/ferriskey-api-abyss/src/federation/mod.rs:15-35`). A tab with no data
  source is not migratable.
- **No "failed accounts" section.** Same reason: `errors` never crosses the DTO
  boundary (D8).
- **No `vendor` column, no imported-account count, no sparklines.** The
  prototype's `vendor` and `importedUsers` are fictional fields, and FK-12
  forbids a series without measured data.
- **The synchronisation settings live in the sync tab, not the create page.**
  The prototype's create page only sets the interval; the create payload
  requires `sync_enabled` and `sync_mode`, so both are asked for at creation
  rather than being silently defaulted.
