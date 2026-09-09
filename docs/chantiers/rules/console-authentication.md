# Business rules inventory — `console/authentication`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

The current section is `front/src/pages/console-authentication/page-console-authentication.tsx`,
30 lines. It carries three route shapes and almost no logic: `identity-providers`
mounts the admin page whole, `sign-in-methods` and `password-policy` are
`ConsoleComingSoon` placeholders. So the inventory below has two halves — what
the current file says (A), and what the screens it mounts or promises say (B).

## A. `page-console-authentication.tsx`

| # | Rule | Carried over |
|---|---|---|
| A1 | Index redirects to `sign-in-methods`, not to the first alphabetical route | ✅ verbatim |
| A2 | `identity-providers/*` is a splat: the admin sub-routes (`create`, `:alias`) live inside the console | ✅ verbatim, now mounting `next/pages/iam/identity-providers/page-identity-providers` |
| A3 | Identity providers is the **same** screen as the admin console, not a copy | ✅ mounted, not cloned — see divergence 1 for the cost |
| A4 | `sign-in-methods` is announced as "Configure passkey, magic link, password and MFA options for this realm." | ✅ built; "password" drops out — see rule S1 |
| A5 | `password-policy` is announced as "Set strength requirements and rotation rules for passwords." | ✅ built; both halves are present (requirements + `max_age_days`) |

## B1. Sign-in methods — built from the domain, no current screen

There is no existing view. The rules come from the fields the server accepts
(`Schemas.UpdateRealmSettingValidator`, `Schemas.RealmSetting`) and from what
the login flow does with them.

| # | Rule | Carried over |
|---|---|---|
| S1 | Password is **not** a toggle. No `password_enabled` field exists in `RealmSetting`; a password is always accepted | ✅ FK-40 — not offered. The section is titled "Passwordless methods" and its description says a password always remains accepted |
| S2 | `login_aliases` is an **ordered** multi-select of `username` / `email`, at least one | ✅ `OrderedChoiceCards`, same `minSelectedReason` shape as the admin login tab, plus an inline error |
| S3 | `passkey_enabled` — boolean | ✅ `SwitchField` (FK-27) |
| S4 | `magic_link_enabled` — boolean | ✅ `SwitchField` |
| S5 | `magic_link_ttl` is shown **only** when magic link is on (the admin login tab does the same) | ✅ conditional `FieldRow`, minutes, `min={1}` |
| S6 | `require_mfa` — boolean. Enforced by `libs/ferriskey-trident/src/mfa_policy.rs:10`: MFA is required when the realm flag **or** any assigned role's `require_mfa` is set | ✅ `SwitchField`. The role half is stated in the description ("Accounts with no second factor enrolled are sent through the enrolment screen") but the per-role override is **not** editable here — that belongs to User management → Roles |
| S7 | `user_registration_enabled`, `email_verification_enabled`, `forgot_password_enabled`, `remember_me_enabled` — booleans, all four affect what the sign-in page renders | ✅ grouped under "Account access", FK-22 descriptions written from the signer's point of view |
| S8 | `lockout_threshold` / `lockout_duration_seconds` — integers, defaulted to 10 / 900 by `core/src/infrastructure/realm/mappers/realm_setting_mapper.rs:75-76`, read at sign-in by `core/src/domain/authentication/services.rs:2247-2287` | ✅ added — see divergence 3, they are surfaced nowhere else in either console |
| S9 | Threshold 0 means "never lock" | ✅ stated in the description and echoed in the footer summary |
| S10 | A threshold above 0 with a duration of 0 locks the account forever | ✅ validation error `A lockout with no duration never releases the account` — announced before the save, not after |
| S11 | The settings endpoint is a partial `PUT`: the repository only writes the `Some(...)` fields (`core/src/infrastructure/realm/repositories/realm_postgres_repository.rs:312-318`) | ✅ the body carries only the eleven fields this screen owns; realm token lifetimes, `compass_enabled`, `seawatch_*` are untouched |
| S12 | Magic link, email verification and password recovery all send mail, and mail needs an SMTP configuration | ✅ FK-39-style: an amber banner appears as soon as one of the three is on and `GET /realms/{realm}/smtp-config` returns nothing, and it names where to fix it. Announced **before** the save |
| S13 | A realm with no `settings` renders a named empty screen rather than a form over defaults | ✅ |
| S14 | The draft is derived at render time from `${settings.id}:${settings.updated_at}`, never mirrored through `useEffect` | ✅ `useDraft` reused from `next/pages/iam/realm/feature/use-draft.ts` |

## B2. Password policy — the admin tab, re-hosted

**Decision: same screen, not a restatement.** The nine fields of
`PasswordPolicy` are the whole policy; there is no customer-facing subset of
them, and inventing softer copy for the same nine booleans would produce two
descriptions of one thing that could drift. So
`next/pages/iam/realm/ui/realm-password-policy-tab.tsx` is imported and
rendered as-is, and the console adds only what a standalone page needs that a
tab inside Realm Settings did not: its own title, its own save bar, and a
customer-facing appendix.

| # | Rule | Carried over |
|---|---|---|
| P1 | Nine fields: `min_length`, four `require_*` booleans, `max_age_days`, `min_entropy_bits`, `forbid_common`, `check_breached` | ✅ the admin tab, unmodified |
| P2 | Validation is `updatePasswordPolicyValidator` from `@/pages/realm/validators` — `min_length` 1…128, `min_entropy_bits` 0…256, `max_age_days` ≥ 0 | ✅ reused verbatim, errors mapped per field |
| P3 | `max_age_days` is nullable server-side; the admin tab coerces `null` to `0` on read | ✅ same coercion, so "0 to disable" stays true |
| P4 | Entropy help text cites CNIL deliberation 2022-100 | ✅ inherited unchanged from the shared tab |
| P5 | `check_breached` says it has no effect without an external provider | ✅ inherited unchanged |
| P6 | Loading renders a skeleton; a failed load renders a named empty screen, not an empty form | ✅ both, at page level now instead of tab level |
| P7 | The save bar only appears when something changed, and counts the changes | ✅ per-field count |
| P8 | Save is refused while the zod parse fails | ✅ `canSave`, and the button falls back to the `secondary` variant |
| P9 | Success and failure both raise a toast | ✅ same two toasts as the admin feature |
| P10 | The public endpoint `/password-policy/public` returns **five** of the nine fields (`libs/ferriskey-api-realm/src/handlers/get_public_password_policy.rs:14-22`), and it is what `pages/authentication/feature/page-register-feature.tsx:48` and `page-reset-password-feature.tsx:29` render next to the field | ✅ added as a "What the person sees" section — see divergence 2 |

## Rules deliberately not carried over

| # | Rule | Why |
|---|---|---|
| A4 | The word "password" in the sign-in-methods promise | FK-40. There is no server field to toggle it. Saying so in the section description beats an inert switch |
| S6 | Per-role `require_mfa` | Out of this section. It is a property of a role and it is edited under User management → Roles; duplicating it here would create two writable copies of one flag |

## Deviations

| Screen | What differs | Why |
|---|---|---|
| Sign-in methods | Five sections instead of the admin's two, and no tabs | The console gives sign-in a whole page where the admin gave it a tab inside Realm Settings; the extra room goes to grouping, not to bigger type |
| Sign-in methods | A footer strip restates the four decisions in one line each | The page is long enough that the top of it scrolls away; the strip is the answer to "what does the sign-in page look like right now" |
| Sign-in methods | `login_aliases` is titled "Identifiers", not "Login identifiers" | The console never says "login" in a noun; the sign-in page is a sign-in page throughout |
| Password policy | The shared admin tab is imported rather than restated | See B2 |
| Password policy | An appendix lists which rules the sign-up page announces and which it enforces silently | P10 |
| Identity providers | The admin screen is mounted whole | A3, and the mission's instruction to mount rather than clone |

## UI ↔ domain divergences found

**1. `require_mfa` is writable by the API and editable in no console.**
`libs/ferriskey-api-realm/src/validators.rs:80` accepts it,
`libs/ferriskey-api-realm/src/handlers/update_realm_setting.rs:73` forwards it,
`libs/ferriskey-trident/src/mfa_policy.rs:10` enforces it — and neither
`front/src/pages/realm/` nor `next/pages/iam/realm/ui/realm-login-tab.tsx`
offers it. The realm flag has been reachable only by hand-written HTTP. This
console now edits it; the admin console still does not. Reported, not fixed —
`next/pages/iam/` is read-only for this chantier.

**2. Four of the nine password rules are enforced but never announced.**
`min_entropy_bits`, `forbid_common`, `check_breached` and `max_age_days` are
absent from `PublicPasswordPolicy`
(`libs/ferriskey-api-realm/src/handlers/get_public_password_policy.rs:14`), so
the sign-up and reset screens cannot list them: the person types a password,
submits, and only then learns it was too weak. Not a bug in the front — the
endpoint decides — but the operator setting `min_entropy_bits: 80` deserves to
know the person will never see the requirement. Surfaced in the appendix
(P10), not fixed.

**3. Account lockout exists in the domain and in no screen.**
`lockout_threshold` and `lockout_duration_seconds` are columns
(`core/src/entity/realm_settings.rs:37-38`), defaulted to 10 / 900
(`realm_setting_mapper.rs:75-76`), read on every failed sign-in
(`core/src/domain/authentication/services.rs:2247-2287`) and accepted by the
update validator — and neither console shows them. Every realm therefore locks
accounts after 10 failures for 15 minutes without anyone having chosen it.
Surfaced here (S8).

**4. `RealmLoginSetting` cannot answer this screen.**
`GET /realms/{name}/login-settings` returns a projection that drops
`require_mfa`, `user_registration_enabled` is present but `lockout_*` are not
(`core/src/domain/realm/entities.rs:11-24`). The screen reads
`GET /realms/{name}` and uses `realm.settings` instead. Worth noting because
the admin login tab reads the projection and so structurally *cannot* grow the
three fields above without a backend change.

## Lines needed in read-only files

See the report — the identity-providers mount navigates out of the console and
needs a base-path change in `front/src/next/routes.ts` plus three call sites in
`front/src/next/pages/iam/identity-providers/feature/`.
