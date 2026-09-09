# Business rules inventory — `seawatch`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Sources: `front/src/pages/seawatch/feature/page-overview-feature.tsx`,
`ui/page-overview.tsx`, `ui/security-metrics.tsx`,
`ui/strange-events-analysis.tsx`, `ui/flagged-users.tsx`.

## `feature/page-overview-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| S1 | When the API returns no event, five hard-coded events are displayed instead | ❌ **deliberately dropped.** A security journal that invents rows is worse than an empty one: the five mock events named plausible actors and IP addresses, and nothing in the table said they were fake. An empty realm now renders the empty state, which says what will fill the list. Reported as a divergence, and since fixed in the current console too — commit `e442b833` |
| S2 | A `Mock data` / `Live feed` badge tells which of the two is on screen | ❌ dropped with S1 — with no mock path left, the badge always reads `Live feed` and carries no information |
| S2b | The query sends no window, no limit and no ordering | ❌ dropped. `useGetSecurityEvents` sends an empty query object, so the server applies `DEFAULT_LIMIT = 100` (`libs/ferriskey-api-seawatch/src/handlers/get_security_events.rs:25`) over the realm's entire history. Every figure the old band showed — total, distinct actors, success rate — was computed over whatever those 100 rows happened to be, under no stated window. The `/next` feature pins the window explicitly; see S42 |

## `ui/security-metrics.tsx`

| # | Rule | Carried over |
|---|---|---|
| S3 | Four metrics: total events, failed events, distinct actors, last event | ⚠️ `MetricsBand` carries `Events`, `Failures`, `Success rate` and `Distinct actors` — the prototype's four. `Last event` survives as the `Events` hint (S47), and the success rate that was S6's subtitle becomes a cell |
| S4 | Counts are locale-formatted (`toLocaleString`) | ✅ |
| S5 | Distinct actors counts `actor_id ?? 'unknown'` — an unattributed event still counts as one actor | ✅ predicate copied verbatim |
| S6 | The failure metric carries `<n>% success rate` as its subtitle, `0%` when there is no event | ✅ promoted to the `Success rate` cell, still `0%` with no event; the `Failures` hint now names the dominant `error_code` as the prototype does |
| S7 | `Last event` is the **maximum** timestamp, not the first row; no event → `No activity yet` | ✅ same reduction, same literal, now rendered as the `Events` hint (S47); with no event the hint names the window instead |
| S8 | Loading shows `...` in place of every figure | ⚠️ `MetricsBand` has no loading variant; during loading the band shows zeroes while `DataView` shows skeleton rows. Reported, not worked around |

## `ui/page-overview.tsx`

| # | Rule | Carried over |
|---|---|---|
| S9 | Events are sorted by timestamp, most recent first | ✅ sorted in the feature |
| S10 | Status filter with three positions: All / Success / Failure | ✅ `Failures` filter, plus three domain families (Authentication, Credentials, Administration) |
| S11 | Free-text search covers `event_type`, `actor_id`, `target_id`, `target_type`, `resource`, `ip_address`, `user_agent`, `status` | ✅ `searchIn`, same eight fields |
| S12 | Filter and search compose | ✅ |
| S13 | `isError` renders a destructive alert `Security events unavailable` | ✅ alert line, tone `error`; the "showing cached or mocked data" half of the sentence is gone with S1 |
| S14 | Loading renders skeleton rows | ✅ `DataView loading` |
| S15 | No match renders a dedicated empty state | ✅ the page distinguishes "nothing exists" from "the filter hides everything" (FK-13) |
| S16 | The event type is humanised for display | ✅ through an explicit label catalogue rather than title-casing, with the raw `event_type` in mono underneath (FK-35) |
| S17 | A failure row is visually distinct from a success row (red lock icon / red badge) | ✅ `Pill` danger + `IconTile` danger — form *and* colour (FK-30) |
| S18 | The actor label falls back `actor_id → target_id → 'Unknown actor'` | ✅ named absence (FK-32) |
| S19 | `resource` is shown as a badge only when present | ✅ shown in the `Target` column, `none` when absent |
| S20 | `target_id` is shown only when present | ✅ same column |
| S21 | `ip_address` is shown only when present | ✅ column, `unknown` when absent |
| S22 | The device icon is a phone when the user agent mentions iphone / android / mobile, a monitor otherwise | ❌ dropped. A three-substring test over a raw user agent is a guess, and the domain stores the header verbatim; the user agent is searchable and shown on the card instead of being turned into an icon that can be wrong |
| S23 | `details.reason` is displayed, with `details.error_code` as a badge next to it | ✅ hoisted into the row, under the status pill (FK-31) |
| S24 | Every other `details` entry is rendered as `Humanised Key: <code>value</code>` | ⚠️ kept in card view only (card footer). A table row cannot carry an arbitrary key/value map, and `/next` has no security-event detail route to move it to. Reported below |
| S25 | The event stream and the two side panels sit in a 2/3 – 1/3 grid | ⚠️ replaced by the prototype's block order: alerts, metrics band, authentication-traffic section, event-stream section — each full width |
| S26 | Realm name shown as a badge next to the page title | ⚠️ dropped, as in the `role` pilot: the shell breadcrumb names the realm |

## `ui/flagged-users.tsx`

| # | Rule | Carried over |
|---|---|---|
| S27 | Only `status === 'failure'` events feed the risky-actor list | ✅ |
| S28 | Events are grouped by `actor_id`, absent → `Unknown actor` | ✅ |
| S29 | The retained IP is the one of the **most recent** failure of that actor | ✅ same reduction |
| S30 | Actors are sorted by failure count, descending, capped at 4 | ⚠️ capped at 3, because they are now alert lines above the table rather than a panel |
| S31 | An actor above 3 failures is shown in `destructive`, below in `secondary` | ✅ tone `error` above 3, `warn` at or below |
| S32 | Empty → `No risky actors detected.` | ⚠️ no alert at all when there is nothing to flag: FK-14's band exists to carry anomalies, and a line saying "no anomaly" is a line of nothing |
| S33 | The panel shows an avatar with computed initials and a cleaned display name | ❌ dropped with the panel — the identifier is shown verbatim, which is what one copies into a search |

## `ui/strange-events-analysis.tsx`

| # | Rule | Carried over |
|---|---|---|
| S34 | Top 5 event types by count, with a percentage bar | ❌ **dropped.** The ported kit exports no bar-list component, and the prototype has no such panel either. (The earlier reason — "`ListingPage` has no slot" — no longer applies: the page composes its sections directly since S45.) The `Event` column is sortable and the table footer aggregates the totals, which answers "which type dominates" on the same screen. Reported as a gap |
| S35 | A `<n> failures` badge in the panel header | ✅ carried into the `Failures` metric |
| S36 | The advice line "Focus on the top 5 event types to reduce noise." | ❌ dropped with S34 — it is a comment, not a fact about this realm |

## New behaviour, from the prototype

| # | Rule | Source |
|---|---|---|
| S37 | No create action, in either empty state: an administrator does not fabricate a security event | FK-15 |
| S38 | ~~Metrics are counts, without sparklines — the prototype's series are invented and no endpoint returns a security-event history.~~ **Corrected.** The first half was wrong and the second half was wrong twice over. A measured history exists: `GET /realms/{realm}/compass/v1/activity/daily` returns one row per day (`date`, `logins`, `login_failures`, `unique_login_users`, `signups`, `total_flows`), computed in SQL over `compass_flows` (`core/src/infrastructure/compass/repositories/compass_flow_postgres_repository.rs:285-345`), wrapped by `useGetDailyActivityStats` (`front/src/api/compass.api.ts:54`). And the loaded security events themselves carry timestamps, so bucketing them per day is a measured series too. See S44 and S45 | FK-12 |
| S44 | The four metric cells carry a sparkline derived **from the security events of the window**, bucketed per day: `Events`, `Failures`, `Success rate`, `Distinct actors`. All four plot whenever the window was measured, **including a flat line at zero**: a realm that recorded no failure on each of the last seven days measured seven zeros, and that flat line is the finding. FK-12 forbids *inventing* a series so a card looks alive, not drawing a real zero — suppressing a measured zero makes the card say less than the data does. The guard is therefore a length test (`series.length > 0`), not a distinct-value test | FK-12, prototype |
| S44a | Two guards remain, because they are about data validity rather than about flatness: **(a)** no series at all when the window is truncated at 500 — under the cap the per-day counts are simply wrong, the oldest days being clipped, so no series is the honest answer; **(b)** `Success rate` additionally requires every day of the window to hold at least one event, since a rate over a day with no event is undefined, not 0 % | FK-12 |
| S44b | The compass daily series is **not** used for the cells. It measures `compass_flows` — authentication flows — while the cells count `security_events`. Plotting one under the other's number would be the same lie FK-12 forbids, in a subtler form. It carries the chart instead, which is titled for what it measures | FK-12 |
| S45 | `Authentication traffic` section, full width, between the metrics band and the event stream: `ActivityChart` from the kit (the same component the overview mounts), fed by `useGetDailyActivityStats` over the same 7-day window, gated on `realm.settings.compass_enabled` exactly as the overview gates it. The section renders **nothing** — no empty frame — when the series is unavailable, holds a single point, or is entirely zero. The header carries the window totals and the Successes / Failures legend the prototype shows | prototype, FK-12 |
| S46 | The event stream sits in its own `Section` titled `Event stream`, whose header carries the filter tabs and the search box, as in the prototype. The sentence stating that the families are server-side and the search only narrows what was loaded is the section description | prototype |
| S47 | `Last event` is no longer a cell of its own: it is the `Events` hint (`last <timestamp>`), which frees the fourth cell for `Success rate` and aligns the band with the prototype's four | prototype |
| S39 | Failures are announced in the alert band, with the dominant failing event type and the actors behind it | FK-14 |
| S40 | The human label carries the row, the raw `event_type` underlines it in grey monospace | FK-35 |
| S41 | No description repeating the label under it | FK-36 |
| S42 | The feature queries an explicit window — `from_timestamp` / `to_timestamp` over the last 7 days, `limit: 500` — and every figure names it: the page description, the `Events` hint and the `Distinct actors` hint all read `last 7 days` | orchestrator decision, following the same reasoning as S1 |
| S43 | When the window holds more events than the limit, an amber alert says so: the figures cover the 500 most recent only. The server orders by `timestamp DESC` before applying the limit (`security_event_postgres_repository.rs:141-145`), so "most recent" is accurate and not a guess | FK-14 |

## Deliberate divergences from the prototype

- ~~The prototype's `TrafficChart` is not exported by the ported kit, and no
  endpoint returns a time series.~~ **Wrong, corrected.** The kit exports
  `ActivityChart` (`front/src/components/kit/index.ts:20`), and the series comes
  from `GET /realms/{realm}/compass/v1/activity/daily`. The chart is mounted;
  see S45. What remains true is only that the series measures *authentication
  flows*, not security events — hence the section title and description name
  Compass, and the chart is hidden when Compass tracing is off.
- `Pause / Resume` and `Export` buttons: neither exists. `useGetSecurityEvents`
  does not poll, so there is no live stream to suspend, and no export endpoint
  is defined for seawatch. An inert control reads as a breakage (FK-37).
- `Bloquer la source` on the failure alert: there is no blocking capability in
  the domain. The alert carries no verb rather than a verb that does nothing.
  Its wording follows the prototype otherwise: the dominant failing event, the
  dominant `details.error_code`, and the dominant `resource` — the nearest thing
  the domain has to the prototype's "client"; see divergence 4 below.
- The layout no longer goes through `ListingPage`: that component renders
  header, alerts, metrics, toolbar and table as one block with no slot between
  the band and the table, so neither the chart (S45) nor the titled event-stream
  section (S46) can be mounted through it. The page composes `MetricsBand`,
  `Section` and `DataView` directly, as `next/pages/overview` does, and keeps
  every `ListingPage` behaviour: alert lines, filter tabs, local search, list /
  card toggle, footer aggregates, the count line, and FK-13's distinction
  between "nothing exists" and "the filter hides everything".
- `Section` exposes one action slot, on the right of its title. The filter tabs
  therefore sit next to the search box on the right rather than immediately
  after the title as in the prototype's own panel header. Widening `Section`
  would mean writing in the frozen kit.
- The prototype's `login / token / logout / error` event families do not exist.
  `SecurityEventType` has 25 variants
  (`libs/ferriskey-seawatch/src/entities.rs:12-92`); the filters group them
  into Failures, Authentication, Credentials and Administration.

## Divergences UI ↔ domain — reported, not fixed

**1. The console displayed fabricated security events. — fixed since.**

`front/src/pages/seawatch/feature/page-overview-feature.tsx:9-63` substituted
five hard-coded events — with real-looking actors (`john.doe@company.com`),
IP addresses and user agents — whenever the API returned an empty list. The
domain hash-chains every event (`event_hash` / `prev_hash`,
`libs/ferriskey-seawatch/src/hashing.rs`) precisely so that the journal cannot
be tampered with; the front then wrote rows into it that the chain never saw.
The `/next` view never did this, and the current console no longer does either:
the fallback and its badge were removed in commit `e442b833`.

**1b. `useGetSecurityEvents` cannot express a window.**

`front/src/api/sea-watch.api.ts:10` sends `query: {}`. The endpoint accepts
`from_timestamp`, `to_timestamp`, `limit`, `offset`, `actor_id`, `client_id`,
`ip_address` and `event_types`, and caps a page at `MAX_LIMIT = 1000` with
`DEFAULT_LIMIT = 100`. The `/next` feature therefore calls
`window.tanstackApi.get` directly rather than through the shared hook: widening
the hook's signature would touch a file the current console also consumes, which
is out of this workstream's scope. The line that would remove the duplication is
reported below.

**2. `SecurityEvent.details` is `unknown` on the wire and has no schema.**

`libs/ferriskey-seawatch/src/entities.rs` types it as free-form JSON, so the
front can only guess at `reason` and `error_code` — which is exactly what
`page-overview.tsx:261-289` does, with a chain of `in` narrowings. Nothing
guarantees those keys exist for a given event type. Until the domain names the
shape per event type, any rendering of `details` is a best effort.

**3. `actor_type` is never shown.**

The domain distinguishes `user`, `service_account`, `admin` and `system`
actors (`ActorType`), which is what tells an operator whether a 3 a.m. client
deletion came from a human or from a pipeline. Neither the current console nor
the prototype surfaces it. The new view shows it as a mono suffix on the actor;
it was the one field with an obvious, cheap home.

**4. A security event names no client.**

The prototype's failure alert reads "all on `admin-cli`, reason
`invalid_grant`". `SecurityEvent`
(`libs/ferriskey-seawatch/src/entities.rs:239-258`) has no `client_id`: it
carries `actor_id`, `actor_type`, `target_type`, `target_id`, `resource` and
free-form `details`. The list endpoint *does* accept a `client_id` filter
(`libs/ferriskey-seawatch/src/value_objects.rs:8`), so the column exists on the
query side but is never returned to the front. The alert therefore names the
dominant `resource` — falling back to `target_id` when `target_type` is
`client` — which is the nearest thing the wire form carries, and the dominant
`details.error_code`, which is exactly the prototype's `invalid_grant`. If the
client is meant to be attributable on a security event, the field is missing
from the response schema.

**5. Two populations, two panels.**

`compass/v1/activity/daily` counts rows of `compass_flows`
(`compass_flow_postgres_repository.rs:296-306`); `seawatch/v1/security-events`
returns rows of `security_events`. A login can produce both, but the two are
written by different paths and neither is derived from the other, so their
totals need not agree. The screen keeps them apart: the chart is titled
`Authentication traffic` and says Compass in its description, the metric cells
and the table are security events only. Nothing on the page adds one to the
other.

## Line that would remove a duplication — reported, not written

`front/src/api/sea-watch.api.ts` is shared with the current console, so this
workstream does not widen it. The `/next` feature would call the shared hook
instead of `window.tanstackApi` if `useGetSecurityEvents` accepted the query
the endpoint already documents:

```ts
export const useGetSecurityEvents = ({
  realm,
  fromTimestamp,
  toTimestamp,
  limit,
}: BaseQuery & { fromTimestamp?: string; toTimestamp?: string; limit?: number }) => {
```

with `query: { from_timestamp: fromTimestamp, to_timestamp: toTimestamp, limit }`.
Every existing caller keeps its behaviour, since all three are optional.

## Not built, deliberately

A window selector (24 h / 7 d / 30 d) would need a control in the listing
toolbar, and `ListingPage` exposes no slot for one — mounting it above the
component would sit outside the page container and break the block order the
style fixes. The window is stated instead of being made adjustable.
