# Business rules inventory — `realm-settings`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Source screens: `front/src/pages/realm/ui/page-realm-settings*.tsx`,
`front/src/pages/realm/layouts/realm-settings-layout.tsx`, and the matching
`feature/` files (the current split leaks data rules into the features, so they
are inventoried here too).

Webhooks were a tab of this tree; their rules live in
`docs/chantiers/rules/webhook.md`.

## `layouts/realm-settings-layout.tsx` + `ui/page-realm-settings.tsx`

| # | Rule | Carried over |
|---|---|---|
| G1 | The active tab is read from the **last path segment**, and falls back to `general` when the segment is not a known tab | ✅ `useRouteTabs` (FK-16) — same fallback, `tabs[0]` is `general` |
| G2 | Nine tabs: General, Login, Tokens, SMTP, Email, Password Policy, Webhooks, Maintenance, Security | ⚠ five kept, two moved, one dropped, one promoted — see the tab-set note below |
| G3 | `Security` is rendered disabled and cannot be opened | ❌ dropped — see the tab-set note |
| G4 | The layout renders nothing at all when the realm is not in `useRealmStore().userRealms` | ✅ dedicated "Realm not found" panel instead of a blank screen (FK-32: the absence is named) |
| G5 | Page header shows the **realm name** as title and the realm `id` in a mono badge | ✅ title + mono `id` in the header metadata block |

**Tab-set note.** The `/next` console keeps General, Login, Tokens, Password
Policy and Maintenance, matching `../ferriskey-kit/src/pages/RealmSettingsPage.tsx`.

- `SMTP` and `Email` **move to the `email-templates` section**, by orchestrator
  decision: the template assignment sits beside the templates it assigns, and
  the SMTP configuration becomes the second tab of that page
  (`Templates | SMTP`), as in `../ferriskey-kit/src/pages/emails/`. Their rules
  are still inventoried below, marked *moved*, so the handover can be checked.
- `Security` carried no screen: `feature/page-realm-settings-security-feature.tsx`
  renders the literal `<div>Security</div>` behind the `REALM_SETTINGS` feature
  flag, and the tab was disabled in the tab bar. Dropped as dead.
- `Webhooks` becomes its own top-level section (FK-20).

## `ui/page-realm-settings-general.tsx` + its feature

| # | Rule | Carried over |
|---|---|---|
| G6 | `name` is rendered **disabled** — the technical name is immutable | ✅ disabled input, mono, with "Immutable after creation" help text |
| G7 | `display_name` is optional, max 255 characters | ✅ `updateRealmValidator` reused verbatim; the error renders under the field |
| G8 | An empty/whitespace `display_name` is sent as `null`, not `''` | ✅ `displayName.trim() ? … : null` copied verbatim |
| G9 | `default_signing_algorithm` is rendered **disabled**: the console never sends it | ✅ disabled select; it now shows the realm's real `settings.default_signing_algorithm` instead of a hard-coded `RS256` — see divergences |
| G10 | The signing-algorithm select lists `HS256`, `RS256`, `ES256` from `SigningAlgorithm` | ⚠ the disabled control shows the realm's current value; the discovery document only announces `RS256`, so offering three choices in a disabled control was noise. Reported. |
| G11 | Delete realm is disabled when the realm is `master`, and the description gains "The master realm cannot be deleted." | ✅ verbatim |
| G12 | Delete requires typing the realm name (`confirmText={realmName}`) | ✅ `DangerZone` unchanged |
| G13 | After deletion, navigate to `/realms/master/overview` | ✅ verbatim |
| G14 | Save toasts `Realm updated successfully.` | ✅ |
| G15 | The save bar appears only when the form differs from the loaded realm | ✅ dirty count computed from the pristine values |
| G16 | Cancel resets the form to the loaded values | ✅ `Discard` |

## `ui/page-realm-settings-login.tsx` + its feature

| # | Rule | Carried over |
|---|---|---|
| L1 | Six boolean capabilities: user registration, email verification, forgot password, remember me, passkey, magic link | ✅ six `SwitchField` in a `Capabilities` section |
| L2 | Each switch is labelled `Enabled` / `Disabled` next to the control | ✅ `SwitchField` defaults are exactly those labels |
| L3 | The **Magic Link TTL** field is rendered only when `magicLink` is on | ✅ conditional row |
| L4 | Magic Link TTL is a number, minimum 1, expressed in **minutes** | ✅ `min={1}`, unit suffix `minutes`, `tnum` (FK-05). Confirmed by `core/src/entity/realm_settings.rs` → `magic_link_ttl_minutes` |
| L5 | Login aliases: only `username` and `email`, in that fixed order | ✅ `OrderedChoiceCards` with the two choices in that order |
| L6 | Toggling **on** re-inserts the alias at its position in the canonical order (`ORDER.filter(...)`), it does not append | ⚠ `OrderedChoiceCards` appends and shows the rank, because the field's own help text says "Order sets precedence" — an append that cannot be reordered contradicts the current re-insert. Reported: the current UI's help text and its behaviour already disagree. |
| L7 | The last remaining alias cannot be unchecked (`if (next.length > 0)`) | ✅ `OrderedChoiceCards` refuses to drop the last one and explains why on hover |
| L8 | The zod schema also enforces `min(1)` with the message `Select at least one login identifier` | ✅ same message, rendered under the field |
| L9 | `login_aliases` falls back to `['username']` when the API returns none | ✅ verbatim |
| L10 | Save sends the whole login block in one `PUT /realms/{name}/settings` | ✅ |

## `ui/page-realm-settings-tokens.tsx` + its feature

| # | Rule | Carried over |
|---|---|---|
| T1 | Four lifetimes: access, refresh, ID, temporary | ✅ same order |
| T2 | All four are seconds, edited through the product's `DurationInput` | ✅ `components/ui/duration-input.tsx` reused (FK-33) |
| T3 | Temporary token help text names its use: "e.g. password reset" | ✅ kept |
| T4 | Defaults when the realm has no settings: 300 / 86400 / 300 / 300 | ✅ same defaults |
| T5 | The save bar text is specific to the tab ("unsaved changes in your token settings") | ⚠ one save bar now covers the whole page, so the wording is generic — see the save-bar note |

## `ui/page-realm-settings-password-policy.tsx` + its feature

| # | Rule | Carried over |
|---|---|---|
| P1 | Nine fields, in this order: min length, uppercase, lowercase, number, special, expiry days, entropy bits, forbid common, check breached | ✅ same order |
| P2 | `min_length` 1…128, `min_entropy_bits` 0…256, `max_age_days` ≥ 0 | ✅ `updatePasswordPolicyValidator` reused verbatim |
| P3 | `max_age_days` = 0 means "no expiry", said in the help text | ✅ verbatim |
| P4 | The entropy help text cites CNIL 2022-100 and suggests 80 | ✅ kept |
| P5 | `check_breached` help text says it requires an external provider | ✅ kept, and states that without one the setting has no effect |
| P6 | Loading renders `Loading password policy...`; a failed load renders `Failed to load password policy.` | ✅ skeleton while loading, named panel on failure (FK-32) |
| P7 | Defaults when absent: `min_length` 8, everything else false / 0 | ✅ verbatim |
| P8 | Save toasts success, and surfaces `error.message` on failure | ✅ verbatim |
| P9 | The policy is saved through its own endpoint, `PUT /realms/{realm_name}/password-policy` | ✅ |

## `feature/page-realm-settings-maintenance-feature.tsx`

The current tab has no `ui/` file — the feature renders the markup directly.

| # | Rule | Carried over |
|---|---|---|
| M1 | Two lists: users and roles, in that order | ✅ |
| M2 | Entries are added and removed **immediately**, one mutation per action — no save bar | ✅ kept out of the dirty count |
| M3 | Removal uses the whitelist **entry id**, not the user/role id | ✅ `entryIdMap` equivalent kept |
| M4 | Users show `username` with `email` as sub-label; roles show `name` with `description` | ✅ `EntityPicker` `label`/`sublabel` (FK-28) |
| M5 | The section explains that these entries apply to *any* client of the realm under maintenance | ✅ kept, plus the prototype's info callout stating they add to each client's own list |
| M6 | Empty states: `No users available.` / `No roles available.` | ✅ `emptyHint` on each picker |

## `ui/page-realm-settings-smtp.tsx` + its feature — **moved**

Handed to the `email-templates` workstream (Emails → SMTP tab). Inventoried
here so nothing is lost in the handover; **none of these is implemented by this
workstream.**

| # | Rule | Status |
|---|---|---|
| S1 | The tab has two mutually exclusive modes: a **read-only display** when a configuration exists, an **edit form** when none does | ➡ moved |
| S2 | "A configuration exists" is `!!data && !isError` — a failed fetch counts as absent | ➡ moved |
| S3 | Display mode shows six facts in order: Host, Port, Encryption (**uppercased**), Username, From Email, From Name | ➡ moved |
| S4 | The password is **never** displayed, in either mode after saving | ➡ moved |
| S5 | Form validation: `host` required (`Host is required`), `port` 1…65535, `username` required, `password` required, `from_email` a valid email (`Must be a valid email`), `from_name` required | ➡ moved |
| S6 | `encryption` is one of `tls`, `starttls`, `none`, rendered uppercased in the trigger | ➡ moved |
| S7 | Defaults: port `587`, encryption `tls`, every string empty | ➡ moved |
| S8 | On blur, a `port` that is not a number falls back to the form's default value rather than staying invalid | ➡ moved |
| S9 | Saving uses a plain inline `Save` button at the end of the form — no floating action bar | ➡ moved |
| S10 | Deleting the configuration goes through a `DangerZone` whose confirmation requires typing the literal `delete` | ➡ moved |
| S11 | The delete description states the consequence: password reset and magic links stop working | ➡ moved |
| S12 | After a successful delete the form is reset to the defaults of S7 | ➡ moved |

## `ui/page-realm-settings-email.tsx` + its feature — **moved**

Handed to the `email-templates` workstream (assignment moves onto the Emails
listing). Same caveat: inventoried, not implemented here.

| # | Rule | Status |
|---|---|---|
| E1 | Two sections, in this order: **Email Actions Configuration** (routing) then **Email Templates** (the list) | ➡ moved |
| E2 | Three routed actions, in this order: Reset Password (`reset_password_template_id`), Magic Link (`magic_link_template_id`), Email Verification (`email_verification_template_id`) | ➡ moved |
| E3 | Each action's select lists **only** the templates whose `email_type` matches that action | ➡ moved |
| E4 | The placeholder and the first option are `Not configured`; choosing it assigns `null` through the `__none__` sentinel | ➡ moved |
| E5 | Assigning writes immediately — one `PUT /realms/{name}/settings` carrying that single field. No save bar | ➡ moved |
| E6 | Template rows show the template name plus a badge with the humanised `email_type`, falling back to the raw value when unknown | ➡ moved |
| E7 | Loading renders the literal `Loading templates...` | ➡ moved |
| E8 | Empty state: dashed panel, `Mail` icon, `No email templates configured.`, and a `Create Template` action | ➡ moved |
| E9 | Edit and Create both navigate to `ADMIN_EMAIL_TEMPLATE_BUILDER_URL(realm, id)`, with the literal `new` as the id for a creation | ➡ moved |
| E10 | Delete removes the template **immediately, without any confirmation** — the only destructive action of the realm-settings tree that is not behind `ConfirmDeleteAlert`. Worth fixing in the receiving workstream | ➡ moved, flagged |

**Save-bar note (T5).** The current console has one `FloatingActionBar` per
tab, each with its own wording. `/next` follows the prototype: a single bar,
counting the dirty groups across every tab, and `Save` fires only the
mutations whose group actually changed — the same shape as the role pilot.
Maintenance stays immediate (M2).

## Divergences UI ↔ domain — reported, not fixed

1. **`default_signing_algorithm` is offered but never sent.** The general form
   hard-codes `SigningAlgorithm.RS256` into its values
   (`feature/page-realm-settings-general-feature.tsx`), renders a disabled
   select over the three-variant `SigningAlgorithm` enum of
   `front/src/api/core.interface.ts`, and its submit body carries only `name`
   and `display_name`. The domain field is
   `RealmSetting.default_signing_algorithm: Option<String>`
   (`libs/ferriskey-domain/src/realm/mod.rs`) and `UpdateRealmSettingValidator`
   does accept it. So the console displays a value it does not read and offers
   choices it cannot save.

2. **The realm has no `enabled` flag.** The prototype's
   `RealmSettingsPage.tsx` renders a "Realm activé" switch. No such field
   exists on `Realm` or `RealmSetting` in `libs/ferriskey-domain/src/realm/mod.rs`,
   nor in `Schemas.Realm` / `Schemas.RealmSetting`. Not reproduced.

3. **`magic_link_ttl` default disagrees.** The domain default is 15 minutes
   (`libs/ferriskey-domain/src/realm/mod.rs`, `magic_link_ttl: 15`); the login
   form's `defaultValues` use 5. Harmless in practice — the loaded settings
   overwrite it — but the number shown before the first fetch is wrong. Kept
   at the domain's 15 in `/next`.

4. **Login-alias ordering is stated but not editable.** The field says "Order
   sets precedence", yet the current control re-inserts each alias at its
   canonical index, so the order can never differ from `['username', 'email']`
   or a subset of it. `LoginAliases` is `Vec<LoginAlias>` in the domain, so
   order is meaningful on the wire. Neither behaviour is fixed here; `/next`
   uses the ordered control, which at least makes the stated rule visible.

5. **`lockout_threshold` / `lockout_duration_seconds` and `require_mfa`,
   `edit_username_enabled`, `email_verification_ttl_hours` exist on
   `RealmSetting` and are accepted by `UpdateRealmSettingValidator`, but no
   screen exposes them.** They were absent from the current console and stay
   absent here; the prototype places brute-force lockout in a Security tab
   that has no implementation on either side.
