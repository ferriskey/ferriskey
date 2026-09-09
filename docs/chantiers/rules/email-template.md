# Business rules inventory — `email-template`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Three current screens carry this domain, not one:

- `front/src/pages/email-template/ui/page-email-template-list.tsx` — the CIAM
  listing (`/console/branding/email-templates`);
- `front/src/pages/realm/ui/page-realm-settings-email.tsx` — the **IAM** screen
  (Realm Settings → Email), the one `/next` replaces; it is the only place that
  carries the assignment;
- `front/src/pages/email-template/ui/page-email-template-builder.tsx` — the
  builder, shared by both;
- `front/src/pages/realm/ui/page-realm-settings-smtp.tsx` — the SMTP settings
  (Realm Settings → SMTP), moved onto the Emails page as its second tab: the
  templates say *what* is sent, SMTP says *through what*, and without a server
  no template ever leaves. The `realm` workstream ships neither tab.

## `ui/page-email-template-list.tsx` (CIAM listing)

| # | Rule | Carried over |
|---|---|---|
| E1 | Three email types only, with fixed labels, short labels and icons: `reset_password` → Reset Password / Password / `KeyRound`, `magic_link` → Magic Link / Magic Link / `Link2`, `email_verification` → Email Verification / Verification / `MailCheck` | ✅ `email-types.ts`, one table shared by the three screens |
| E2 | One tone per type (amber, violet, emerald) | ✅ amber / violet / success — `IconTile` and `Pill` tones (FK-02); emerald becomes the kit's `success` |
| E3 | An unknown `email_type` falls back to the first type's metadata | ✅ same fallback, kept although the API enum forbids it |
| E4 | The type counters double as filters; the `All templates` tile clears the filter | ✅ `MetricsBand` for the counts, a segmented type filter in the section header — the metric tiles are no longer clickable (see deviations) |
| E5 | Search matches `name` and `email_type` only | ✅ predicate copied verbatim |
| E6 | Sort by most recent (`updated_at \|\| created_at`, desc) or by name (`localeCompare`); default most recent | ✅ default order kept; the explicit sort control is dropped (see deviations) |
| E7 | Relative dates: `just now`, `Nm ago`, `Nh ago`, `Nd ago`, then a locale date past 30 days; an unparseable date renders `—` | ✅ `formatRelative` copied verbatim |
| E8 | Import opens a hidden `<input type=file accept="application/json,.json">`; the visible button is the affordance | ✅ |
| E9 | The file input is reset after each pick so choosing the same file twice fires `onChange` again | ✅ |
| E10 | The picked file is parsed then posted as-is; the server refuses a foreign or unreadable envelope and its message is surfaced | ✅ `readExportFile` + `useImportEmailTemplate` reused verbatim |
| E11 | Export exists in two formats, JSON and MJML, as two separate actions; failure toasts `Could not export this template` | ✅ |
| E12 | Delete is immediate, without confirmation | ❌ **deliberately dropped.** Deleting an assigned template unassigns it (`ON DELETE SET NULL`) and reverts that email to the plain-text body — FK-42 requires saying so before, not after. Delete now goes through `ConfirmDeleteAlert`, and the confirmation names the emails that lose their template |
| E13 | Loading renders six skeleton cards, never the empty state | ✅ skeleton rows |
| E14 | Two distinct empty states: no template at all (with a create CTA) vs no match for the filters (adjust the filters) | ✅ FK-13 |
| E15 | Create navigates to the builder with the literal id `new` | ✅ |
| E16 | Edit navigates to the builder for that template | ✅ from the row and from the detail page |

## `ui/page-realm-settings-email.tsx` (IAM — the screen `/next` replaces)

| # | Rule | Carried over |
|---|---|---|
| E17 | Assignment section, one row per email action: `reset_password_template_id`, `magic_link_template_id`, `email_verification_template_id`, each with its own description | ✅ `Section` "Assignment", `FieldRow` per action |
| E18 | The candidate list of a row is filtered to the templates whose `email_type` matches that action | ✅ predicate copied verbatim |
| E19 | `__none__` is the sentinel for "Not configured"; picking it sends `null` | ✅ |
| E20 | An assignment is saved immediately on change, one field per request, with no save bar | ✅ unchanged — the save bar on the detail page covers the name only |
| E21 | The assignment section comes **before** the templates list | ✅ order preserved |
| E22 | A template row shows the name and a badge with the type label, falling back to the raw value | ✅ `Pill mono` with the raw `email_type`, plus the human label as the row subtitle (FK-35) |
| E23 | Empty state: dashed panel, `Mail` icon, `No email templates configured.` and a create CTA | ✅ wording aligned with E14 |
| E24 | Loading shows a text placeholder | ✅ replaced by skeleton rows (E13) |

## `ui/page-email-template-builder.tsx` + its feature

| # | Rule | Carried over |
|---|---|---|
| E25 | `template_id === 'new'` switches the builder to creation | ✅ |
| E26 | The template query is disabled when the id is empty or `new`; the loading screen only shows when not creating | ✅ `useGetEmailTemplate` reused verbatim |
| E27 | The stateful builder is remounted through a `key` on the template id, so its initial state is derived, never synchronised in an effect | ✅ same two-component split |
| E28 | The stored structure is read as `structure.children ?? []` and written back as `{ children: tree }` | ✅ verbatim |
| E29 | The email type selector appears **only** while creating: `UpdateEmailTemplateValidator` takes `name` and `structure` only | ✅ and on the detail page the field is disabled *and* says why (FK-23) |
| E30 | The variables of the current type are fetched and handed to the MJML adapter — they feed the variable picker of the text editors | ✅ unchanged |
| E31 | The adapter is rebuilt when the variables change | ✅ same `useMemo` |
| E32 | Save is disabled while a mutation is pending **or** while the name is empty (`name` is `min 1` server-side); its label becomes `Saving...` | ✅ |
| E33 | Creating navigates back to the listing on success; updating stays on the builder | ✅ |
| E34 | A preset sets both the name and the email type | ✅ |
| E35 | Viewport switcher desktop / tablet / mobile driving `PREVIEW_WIDTHS`, desktop by default | ✅ |
| E36 | Back returns to the listing | ✅ to the `/next` listing |
| E37 | Canvas, block library, drag and drop, MJML hierarchy (`allowedChildren`) and validation come from `@/lib/builder-core` + `@/lib/builder-mjml` | ✅ imported unchanged; only the chrome around them is rewritten |

## `ui/page-realm-settings-smtp.tsx` + its feature (SMTP tab)

| # | Rule | Carried over |
|---|---|---|
| S1 | Two modes: with a configuration the screen is **read-only** (six facts plus the danger zone); without one it is a form. Changing a stored configuration is impossible — it has to be deleted and retyped | ❌ **deliberately dropped.** The endpoint is `PUT …/smtp-config`, an upsert (`libs/ferriskey-api-realm/src/handlers/upsert_smtp_config.rs`), so the domain has always allowed the edit; only the screen refused it. The tab is now always editable |
| S2 | Seven fields with their descriptions: Host, Port (`587, 465, 25`), Encryption, Username, Password, From Email, From Name | ✅ wording kept, split into `SMTP server` and `Sender` (FK-25: what identifies the server before what recipients see) |
| S3 | Validation: `host` min 1 `Host is required`; `port` number between 1 and 65535; `username` min 1 `Username is required`; `password` min 1 `Password is required`; `from_email` `Must be a valid email`; `from_name` min 1 `From name is required`; `encryption` one of `tls`, `starttls`, `none` | ✅ the very same schema, imported from `@/pages/realm/schemas/smtp-config.schema` |
| S4 | Defaults: port `587`, encryption `tls` | ✅ |
| S5 | Leaving the port field with a non-number restores the default | ✅ |
| S6 | Three encryption options, rendered uppercase: TLS, STARTTLS, None | ✅ labels kept, each with the consequence of the choice (FK-22) |
| S7 | The password is never displayed: `SmtpConfig` carries no `password` field, the API never returns it | ✅ FK-34 — empty field, `stored, write-only` pill, and a description saying it has to be retyped |
| S8 | Deleting requires typing `delete` in the confirmation | ✅ `confirmText='delete'` unchanged |
| S9 | After a delete the form falls back to the defaults | ✅ derived from the query, not reset by hand |
| S10 | Danger-zone copy: `Email features (password reset, magic links) will stop working.` | ✅ verification added to the list, since it is also gated on SMTP |
| S11 | `hasConfig = !!data && !isError`, with `retry: false` on the query: a 404 means unconfigured, not a transient failure | ✅ `useGetSmtpConfig` reused verbatim |
| S12 | Saving is an explicit button, validation runs on submit | ✅ becomes the sticky `FloatingActionBar`, shown only when the draft differs (FK page skeleton) |
| S13 | Save and delete toasts, including the server's message on failure | ✅ `@/api/smtp.api.ts` reused verbatim |

**Schema, shared not duplicated.** `smtpConfigSchema` now lives in
`front/src/pages/realm/schemas/smtp-config.schema.ts`, on the
`pages/role/schemas/` convention, and both consoles import it. The `/next` tab
imports it through `front/src/next/pages/email-template/smtp-form.ts`, which
adds only what is proper to this screen: the empty draft, the encryption
options with their hints (N8), and the field-by-field error mapping. The
short-lived mirror `smtp-schema.ts` is deleted.

## API layer, reused verbatim

`@/api/email-template.api.ts` and `@/api/builder-export.ts` are imported as-is:
success toasts, the import error toast carrying the server's message, and the
invalidation of the templates list after every mutation.

## Added by this migration

| # | Rule | Why |
|---|---|---|
| N1 | Detail page: the template's type is shown disabled, with the reason — it determines the variables the engine will supply | FK-23 |
| N2 | Detail page: the available variables of the type are listed, each marked as cited or absent from this template | FK-32 |
| N3 | Detail page: a variable cited by the MJML but not supplied for this type raises an amber banner — the send does not fail, the `{{…}}` leaves verbatim in the received email | FK-43. Nothing in the current console says this. The citation is matched **literally**: `interpolate_variables` replaces the exact string `{{key}}`, so `{{ user.email }}` with spaces is never interpolated and is reported as unsupplied |
| N4 | Delete says what it unassigns, in amber, before doing it | FK-42 |
| N5 | The listing warns about email actions with no template assigned | The absence is the information (FK-32); nothing said it before |
| N6 | The SMTP tab is editable even when a configuration exists; the password must be retyped on every save | The endpoint is an upsert and `password` is `min 1` server-side, so an edit is legal but a partial one is not. Saying it beats a screen that forces delete-then-retype (S1) |
| N7 | With no SMTP configuration, an amber banner says nothing is delivered at all | FK-32 / FK-43 in spirit: the silent failure is the whole point of the screen |
| N8 | Each encryption option carries its consequence and its usual port | FK-22 |

## Deviations from the prototype

| Screen | What differs | Why |
|---|---|---|
| Page | Two tabs, `Templates` / `SMTP`, anchored in the URL (`useRouteTabs`, FK-16); the header actions only exist on `Templates` | The prototype's shape, kept as-is. The `realm` workstream drops its `email` and `smtp` tabs |
| Listing | Metric tiles are not clickable filters (E4); the type filter is a segmented control in the `Templates` section header | The prototype separates them the same way, and `MetricsBand` has no click affordance |
| Listing | No explicit sort control (E6) | The prototype has none; most-recent-first is kept as the order |
| Listing | The unassigned-emails banner is at the top, not at the bottom of the page | FK-14: an anomaly belongs to the banner, in the head of the listing |
| Listing | The listing is composed by hand (`MetricsBand` + `Section`) rather than through `ListingPage` | The assignment section must sit between the metrics and the list — FK's own listing order (FK-11) — which `ListingPage` cannot express. Same choice as the prototype |
| Listing / detail | Assignment uses a `Select`, not `EntityPicker` (FK-28) | Three candidates at most, filtered by type; a picker with a search field costs more than it gives. Reported rather than decided for good |
| Detail | An `Export` button (JSON) sits in the header and an `Export MJML` next to the MJML block | The product exports in two formats (E11); the prototype only knew one |
| Detail | The header carries the update date and the id on the right (FK-11) | The prototype showed neither |
| Builder | Its chrome is rewritten; the builder itself is imported unchanged from `@/lib/builder-core` and `@/lib/builder-mjml` | `../ferriskey-kit/src/pages/EmailTemplateBuilderPage.tsx` renders MJML incorrectly. Decision recorded in `front-ferriskey-style-iam.md` |
| Builder | Outside creation, the type is a read-only `Pill` instead of being hidden | FK-23: the value is worth showing; only its editability is refused |
| SMTP tab | No `Send a test email` action | The prototype invents one; **no such endpoint exists** anywhere in the API. FK-40: only offer what the server accepts. Reported as a gap, not built |
| SMTP tab | The password field does **not** say "leave it empty to keep the stored value" | That is the prototype's copy and it is false here — see divergence 4 |
| Route | Creating opens `…/email-templates/create/builder`, not `…/new/builder` | Consistency with `roles/create` inside `/next`. The `'new'` sentinel the builder's query expects is kept internally, never in the URL |

## Divergences UI ↔ domain — reported, not fixed

**1. Without an assigned template, no built-in template is sent.**
The prototype says FerrisKey falls back to "its built-in template". It does
not. `core/src/domain/trident/services.rs:1503` and `:1820` only render a
template when `settings.<type>_template_id` is `Some` *and* an SMTP config
exists; otherwise `html_body` stays `None` and the recipient gets the
hard-coded plain-text body (`"Click the link below to sign in:…"`). Same shape
in `libs/ferriskey-mail/src/email_verification/services.rs:201`. The new screens
say "the plain-text default email", not "the built-in template".

**2. Nothing in the console can grant `manage_email_templates` /
`view_email_templates`.** `libs/ferriskey-domain/src/role/permission.rs`
declares them (bits 25–26); `front/src/api/core.interface.ts` and
`pages/role/types/permission-groups.ts` omit them. Already reported by the
`role` workstream; repeated because this domain is the one they gate.

**3. The MJML of a template is generated server-side and never re-read by the
current console.** `EmailTemplate.mjml` is returned by the API and used by
nothing in `front/src/`. The detail page now shows it read-only, which is also
the only place the cited variables can be computed from (N3).

**4. The prototype's "leave the password empty and the stored one is kept" is
false.** `UpsertSmtpConfigValidator.password` is `#[validate(length(min = 1))]`
(`libs/ferriskey-api-realm/src/validators.rs:92`) and the handler stores what it
receives: an empty password is a 400, not a no-op. Since `SmtpConfig` never
returns the password either, **any** change to the SMTP tab requires retyping
it. The screen says so instead of pretending otherwise.

**5. No endpoint can test an SMTP configuration.** There is no test-email route
in `libs/ferriskey-api-realm` or anywhere else; the only way an administrator
learns the configuration is wrong is a user failing to receive an email. Worth
a domain change, not a UI one.

**6. `email_type` is typed `string` in `CreateEmailTemplateValidator` and
`ImportEmailTemplateValidator`, but `EmailType` everywhere else.** An invalid
string is rejected at `TryFrom<String>`
(`libs/ferriskey-mail/src/email_template/entities.rs:29`) with a 400, not by the
validator. The UI only ever sends the three legal values.
