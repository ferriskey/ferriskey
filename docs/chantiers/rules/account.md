# Business rules inventory — `account`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Sources read in full: `front/src/pages/account/ui/page-account.tsx`,
`front/src/pages/account/feature/page-account-feature.tsx`,
`front/src/pages/account/validators.ts`, `front/src/pages/account/page-account.tsx`.

There is no prototype screen for this page. It is built from `Section` and
`FieldRow`, following the pilot `front/src/next/pages/role/`.

## `ui/page-account.tsx` + `feature/page-account-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| A1 | The page edits **the caller's own profile** (`GET/PUT /realms/{realm}/users/me`), never a user by id | ✅ `useGetOwnProfile` / `useUpdateOwnProfile` reused verbatim |
| A2 | `username` is editable **only** when the realm's `settings.edit_username_enabled` is true | ✅ `disabled={!usernameEditable}` |
| A3 | `edit_username_enabled` is read from `useGetRealm(...)` and defaults to `false` when the realm or its settings are missing | ✅ `?? false` copied verbatim |
| A4 | The username description changes with that setting: `Unique login identifier for your account.` when editable, `Your administrator has disabled username changes for this realm.` when not | ✅ both strings preserved (FK-23 — a disabled field says why) |
| A5 | Fields, in order: username, email, first name, last name | ✅ same order |
| A6 | Validation: `username` required (`Username is required`); `email` a valid address **or** the empty string; `firstname` / `lastname` optional | ✅ `updateOwnProfileValidator` reused verbatim |
| A7 | The save bar appears only when the form differs from the loaded profile | ✅ dirty count derived at render, no `setState` in an effect |
| A8 | Cancel resets the form to the loaded profile | ✅ `onDiscard` |
| A9 | Success toasts `Your profile was updated` | ✅ |
| A10 | Failure toasts `error.message` | ✅ |
| A11 | The page renders `null` while loading or with no profile | ✅ replaced by a skeleton, then a *profile unavailable* panel — a blank screen is indistinguishable from a crash |
| A12 | The email field has no verification affordance, and email verification is not shown here | ✅ kept — `OwnProfileResponse` returns the full `User`, but `UpdateOwnProfileValidator` exposes only `username`, `email`, `firstname`, `lastname`, so nothing else is actionable from this page |
| A13 | No danger zone: a user cannot delete their own account from here | ✅ kept — no endpoint exists for it |

## Divergences UI ↔ domain — reported, not fixed

**1. The account page sends `username` even when the realm forbids editing it.**
`UpdateOwnProfileValidator` accepts `username`, and the form always submits
the whole object — including `username` — regardless of
`edit_username_enabled`. The field being `disabled` means the value is
unchanged, so nothing is corrupted, but the guard is purely client-side: the
setting is a *realm* setting read by the console, and whether
`PUT /realms/{realm_name}/users/me` re-checks it is not visible from the
generated client. Not changed here; carrying the exact current behaviour.

**2. `email_verified` is not reset when the user changes their own email.**
`Schemas.User` carries `email_verified`, and this page can change `email`
without any indication of what happens to that flag. The console shows
nothing about it. Whether the backend clears the flag on an email change is a
domain question this migration does not answer; the screen says nothing
rather than asserting something false.
