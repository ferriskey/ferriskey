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
| S3 | Four metrics: total events, failed events, distinct actors, last event | ✅ `MetricsBand`, same four, each naming the window it covers |
| S4 | Counts are locale-formatted (`toLocaleString`) | ✅ |
| S5 | Distinct actors counts `actor_id ?? 'unknown'` — an unattributed event still counts as one actor | ✅ predicate copied verbatim |
| S6 | The failure metric carries `<n>% success rate` as its subtitle, `0%` when there is no event | ✅ `hint` |
| S7 | `Last event` is the **maximum** timestamp, not the first row; no event → `No activity yet` | ✅ same reduction, same literal |
| S8 | Loading shows `...` in place of every figure | ⚠️ `MetricsBand` has no loading variant; during loading the band shows zeroes while `DataView` shows skeleton rows. Reported, not worked around |

## `ui/page-overview.tsx`

| # | Rule | Carried over |
|---|---|---|
| S9 | Events are sorted by timestamp, most recent first | ✅ sorted in the feature |
| S10 | Status filter with three positions: All / Success / Failure | ✅ `Failures` filter, plus three domain families (Authentication, Credentials, Administration) |
| S11 | Free-text search covers `event_type`, `actor_id`, `target_id`, `target_type`, `resource`, `ip_address`, `user_agent`, `status` | ✅ `searchIn`, same eight fields |
| S12 | Filter and search compose | ✅ |
| S13 | `isError` renders a destructive alert `Security events unavailable` | ✅ `ListingPage` alert, tone `error`; the "showing cached or mocked data" half of the sentence is gone with S1 |
| S14 | Loading renders skeleton rows | ✅ `DataView loading` |
| S15 | No match renders a dedicated empty state | ✅ `ListingPage` distinguishes "nothing exists" from "the filter hides everything" (FK-13) |
| S16 | The event type is humanised for display | ✅ through an explicit label catalogue rather than title-casing, with the raw `event_type` in mono underneath (FK-35) |
| S17 | A failure row is visually distinct from a success row (red lock icon / red badge) | ✅ `Pill` danger + `IconTile` danger — form *and* colour (FK-30) |
| S18 | The actor label falls back `actor_id → target_id → 'Unknown actor'` | ✅ named absence (FK-32) |
| S19 | `resource` is shown as a badge only when present | ✅ shown in the `Target` column, `none` when absent |
| S20 | `target_id` is shown only when present | ✅ same column |
| S21 | `ip_address` is shown only when present | ✅ column, `unknown` when absent |
| S22 | The device icon is a phone when the user agent mentions iphone / android / mobile, a monitor otherwise | ❌ dropped. A three-substring test over a raw user agent is a guess, and the domain stores the header verbatim; the user agent is searchable and shown on the card instead of being turned into an icon that can be wrong |
| S23 | `details.reason` is displayed, with `details.error_code` as a badge next to it | ✅ hoisted into the row, under the status pill (FK-31) |
| S24 | Every other `details` entry is rendered as `Humanised Key: <code>value</code>` | ⚠️ kept in card view only (card footer). A table row cannot carry an arbitrary key/value map, and `/next` has no security-event detail route to move it to. Reported below |
| S25 | The event stream and the two side panels sit in a 2/3 – 1/3 grid | ⚠️ replaced by `ListingPage`'s fixed block order (FK: listing skeleton) |
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
| S34 | Top 5 event types by count, with a percentage bar | ❌ **dropped.** `ListingPage` has no slot for a distribution panel, and the ported kit exports no bar-list component. The `Event` column is sortable and the table footer aggregates the totals, which answers "which type dominates" on the same screen. Reported as a gap |
| S35 | A `<n> failures` badge in the panel header | ✅ carried into the `Failures` metric |
| S36 | The advice line "Focus on the top 5 event types to reduce noise." | ❌ dropped with S34 — it is a comment, not a fact about this realm |

## New behaviour, from the prototype

| # | Rule | Source |
|---|---|---|
| S37 | No create action, in either empty state: an administrator does not fabricate a security event | FK-15 |
| S38 | Metrics are counts, without sparklines — the prototype's series are invented and no endpoint returns a security-event history | FK-12 |
| S39 | Failures are announced in the alert band, with the dominant failing event type and the actors behind it | FK-14 |
| S40 | The human label carries the row, the raw `event_type` underlines it in grey monospace | FK-35 |
| S41 | No description repeating the label under it | FK-36 |
| S42 | The feature queries an explicit window — `from_timestamp` / `to_timestamp` over the last 7 days, `limit: 500` — and every figure names it: the page description, the `Events` hint and the `Distinct actors` hint all read `last 7 days` | orchestrator decision, following the same reasoning as S1 |
| S43 | When the window holds more events than the limit, an amber alert says so: the figures cover the 500 most recent only. The server orders by `timestamp DESC` before applying the limit (`security_event_postgres_repository.rs:141-145`), so "most recent" is accurate and not a guess | FK-14 |

## Deliberate divergences from the prototype

- The prototype's `TrafficChart` is not exported by the ported kit
  (`front/src/components/kit/index.ts` exports `Sparkline` only), and no
  endpoint returns a time series of security events. Dropped rather than fed
  with invented points (FK-12).
- `Pause / Resume` and `Export` buttons: neither exists. `useGetSecurityEvents`
  does not poll, so there is no live stream to suspend, and no export endpoint
  is defined for seawatch. An inert control reads as a breakage (FK-37).
- `Bloquer la source` on the failure alert: there is no blocking capability in
  the domain. The alert carries no verb rather than a verb that does nothing.
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
