# Business rules inventory — `overview`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Source screen: `front/src/pages/overview/ui/page-home.tsx` (441 lines, the
only `ui/` file of the domain) and its feature
`front/src/pages/overview/feature/page-home-feature.tsx`.

## Header

| # | Rule | Carried over |
|---|---|---|
| O1 | Greeting: first token of `user.name`, else `preferred_username`, else nothing | ✅ right end of the identity line |
| O2 | The realm name is shown as a badge next to the identity | ✅ `Pill` primary mono |

## KPI cards → `MetricsBand`

| # | Rule | Carried over |
|---|---|---|
| O3 | Users: value `users.length`, hint `<n>% email verified`, or the literal `No users yet` | ✅ hint lowercased to match the band's register (`no user yet`) |
| O4 | Clients: value `clients.length`, hint `<n> active`, or `No clients yet` | ✅ |
| O5 | Roles: value `roles.length`, hint `Permissions & policies` | ✅ |
| O6 | Auth flows: value `flowStats.total`, hint `<n>% success`, or `No traces yet` | ✅ |
| O7 | The percentage helper rounds and returns `0` when the denominator is `0` | ✅ copied verbatim |
| O8 | The four cards render skeletons while any of the five queries loads | ✅ page-level skeleton |
| O13 | `isLoading` is the disjunction of the five queries (clients, users, roles, compass stats, realm) | ✅ same five queries |

## User growth chart

| # | Rule | Carried over |
|---|---|---|
| O9a | 30-day window, one bucket per day, built from `user.created_at` | ✅ the window is kept, and it is the window the API's daily-activity endpoint uses by default |
| O9b | The window total is displayed as `+<n> new users` | ✅ carried as the `delta` of the Users metric (only when > 0 — the kit renders a delta as a green rise, so a decrease would be shown as a rise) |
| O9c | Empty state `No users yet` when `users.length === 0` | ✅ named absence on the activity panel |
| O9d | The panel plots *new accounts per day* | ⚠️ **replaced.** The panel now plots the server-measured `logins` / `login_failures` daily series returned by `/compass/v1/activity/daily`. Reason: the prototype's overview carries one activity chart, and this series is measured end-to-end by the API (see the FK-12 note below); the per-day signup count survives as the Users delta and as the measured cumulative sparkline of the Users metric. |

## Realm capabilities

| # | Rule | Carried over |
|---|---|---|
| O10 | Six capabilities, in this order: Passkey, Magic links, Self registration, Password reset, Remember me, Compass tracing | ✅ same order, same labels, same descriptions |
| O11 | Each is `!!realmSettings?.<flag>`; a realm without settings shows everything off | ✅ `Boolean(...)`, same behaviour — see the divergence note |
| O12a | Enabled reads `Active`, disabled reads `Disabled` | ✅ rendered as `on` / `off` with the check mark of the prototype (FK-30: shape *and* colour) |

## Quick access

| # | Rule | Carried over |
|---|---|---|
| O12 | Four tiles — Users, Clients, Roles, Compass — with their descriptions verbatim | ✅ pointing at the `/next` routes |
| O14 | Navigation is prefixed by the current realm; a missing realm is a no-op | ✅ the realm falls back to `master`, as in the `role` pilot |

## Added by the prototype, absent from the current screen

| # | Element | Source of truth |
|---|---|---|
| O15 | Inline anomaly band (FK-14) | Computed from real data: failed compass flows, disabled clients, unverified emails when `email_verification_enabled`, Compass tracing off. Each carries a detail and an action verb that navigates somewhere it can be acted on. |
| O16 | Event log | `/compass/v1/flows?limit=8`. Columns: status, grant type, client, account, duration, started. `client_id` / `user_id` are resolved against the clients and users already fetched; unresolved ones read `no client` / `not identified` (FK-32), and a missing duration reads `not measured`. Durations switch unit at one second (FK-33). |
| O17 | `<n> capabilities disabled` in the identity line | Derived from the same six capability flags. |
| O18 | The prototype's `À traiter` checklist and its second `Clients` panel | **Dropped.** Both are backed by mock data with no domain equivalent (a per-realm onboarding checklist does not exist). The right column carries the quick-access tiles of the current screen instead. |
| O19 | The prototype's `Synced 30 s ago · v<version>` and `environment` pill | **Dropped.** Neither a sync timestamp nor an environment nor a version exists on `Realm` (`api.client.ts`, `Realm`). |

## FK-12 — what the API actually exposes as a series

Asked explicitly by the mission. Findings:

**Measured series that exist:**

- `GET /realms/{realm}/compass/v1/activity/daily` →
  `DailyActivityStats[]`, one row per day, zero-filled by the SQL
  (`generate_series`), default window = the 30 days ending today
  (`libs/ferriskey-api-compass/src/handlers/get_daily_activity_stats.rs`).
  Each row carries `signups`, `logins`, `login_failures`, `pending_logins`,
  `expired_logins`, `total_flows`, `unique_login_users`,
  `avg_login_duration_ms`. This is a genuine, server-side measured history —
  the activity chart and the `Auth flows` sparkline use it directly.
- `users`, `clients` and `roles` all carry `created_at`, so a **cumulative
  count per day** over the same 30-day window is a measurement, not an
  invention. The Users / Clients / Roles sparklines are built that way.

**Series that do not exist:** nothing exposes a history of *totals* — no
snapshot table, no per-day count of clients, roles, organizations or
enabled/disabled state. There is no history at all for organizations
(hence no sparkline on that listing).

**Consequence in the code:** a series is attached to a metric only when it
has more than one distinct value (`measured()` in
`feature/page-overview-feature.tsx`). A realm whose four figures never moved
gets no sparkline at all, and `MetricsBand` falls back to its counts
variant — the figure takes the gutter the chart would have used, exactly as
FK-12 asks. The `delta` is passed only when strictly positive, for the same
reason: the kit draws every delta as a green rise.

**Reserve on the `Auth flows` metric:** its value comes from
`/compass/v1/stats`, which aggregates *all* traced flows, while its
sparkline is the 30-day `total_flows` series. The figure and the curve
therefore do not cover the same period. Reported rather than silently
"fixed" by recomputing the total from the window — the current console
already displays this all-time total, and changing it is a product
decision.

## Divergences UI ↔ domain — reported, not fixed

1. **A realm without settings is indistinguishable from a realm with
   everything disabled.** `Realm.settings` is `(null | RealmSetting) |
   undefined` (`front/src/api/api.client.ts`), and both the current screen
   and this one coerce it with `!!` / `Boolean()`. A failed or pending
   settings load therefore reads as "six capabilities disabled". Carried
   over as-is to keep the behaviour identical.
2. **Compass endpoints are queried unconditionally by the current screen.**
   `useGetStats` runs whether or not `compass_enabled` is set. Here the
   *flows* and *daily activity* queries are gated on
   `settings.compass_enabled` (by passing an undefined realm, which turns
   the hooks off), but `useGetStats` is left ungated so the `Auth flows`
   KPI keeps the exact value the current console shows.
3. **The realm title uses `display_name` with the realm name as fallback.**
   `Realm.display_name` is optional in the schema; the current screen never
   displays it at all and shows the raw `realm_name` badge only.
