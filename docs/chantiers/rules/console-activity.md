# Business rules inventory — CIAM console, `activity`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Sources: `front/src/pages/activity/page-activity.tsx`,
`feature/page-live-activity-feature.tsx`, `ui/page-live-activity.tsx`,
`feature/page-logs-events-feature.tsx`, `ui/page-logs-events.tsx`.

Reference for the rebuilt screens: `front/src/next/pages/iam/seawatch/`,
whose own inventory is `docs/chantiers/rules/seawatch.md` (rules `S1`…`S47`).

## `page-activity.tsx` — routing

| # | Rule | Carried over |
|---|---|---|
| A1 | The index route redirects to `live` | ✅ |
| A2 | Four sub-routes in this order: `live`, `logs`, `sessions`, `messages` | ✅ |
| A3 | `sessions` and `messages` render `ConsoleComingSoon` | ⚠️ replaced. Both now render what the API actually measures, and name in prose the part of the promise the API cannot back — see A30 and A38. `ConsoleComingSoon` announced a screen; the new screens announce a *limit*, which is a fact about the product rather than a promise about the roadmap |

## `feature/page-live-activity-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| A4 | The window queried is the last 90 days, `from` = today − 89 days, `to` = today | ✅ same 90-day fetch, same `to.setDate(to.getDate() - 89)` arithmetic, same `YYYY-MM-DD` parameter form |
| A5 | The date parameters are the ISO date only, not a timestamp | ✅ `toDateParam` copied |
| A6 | The window is memoised once per mount, so the query key is stable | ✅ |
| A7 | `activityData?.data ?? []` — an absent response is an empty series, never an error state | ✅ |
| A8 | The query is not gated on anything | ❌ **deliberately dropped.** `compass/v1/activity/daily` counts rows of `compass_flows`, which are only written when Compass tracing is on (`realm.settings.compass_enabled`). With tracing off the endpoint answers zeros, and the old screen drew those zeros as if they were a measurement of the realm. The page now reads the realm settings and, when tracing is off, says so instead of plotting. Same gate the IAM overview and Sea Watch apply (rule S45) |

## `ui/page-live-activity.tsx`

| # | Rule | Carried over |
|---|---|---|
| A9 | Three ranges: `7d` / `30d` / `90d`, labelled `Last 7 days` / `Last 30 days` / `Last 90 days` | ✅ as a `Segmented`, labels shortened to `7 days` / `30 days` / `90 days` since the control sits under a heading that already says which window is on screen |
| A10 | Each of the three blocks (sign-ups card, logins card, combined chart) carries **its own independent** range selector | ❌ **deliberately dropped.** One window governs the page. Three selectors over one series let the screen show `12 sign-ups` next to `340 logins` computed over different windows, with nothing on screen relating the two; every figure on the new page is stated over the same, named window |
| A11 | The visible series is `dailyActivity.slice(-days)` — the tail of the fetched 90 days | ✅ same slice |
| A12 | Totals are the sum over the visible slice, not over the fetched 90 days | ✅ |
| A13 | The day label is `weekday, day` at 7 days and `month, day` beyond | ⚠️ the chart is `ActivityChart` from the kit, which formats its own axis as `month day` at every width (`front/src/components/kit/charts.tsx:72`). Writing a second chart to recover one label format is what the briefing forbids |
| A14 | Loading replaces every figure with a skeleton | ⚠️ `MetricsBand` has no loading variant — same limitation as S8. During loading the band shows zeroes and the chart section is not rendered |
| A15 | The header carries the sentence "You are currently viewing this realm's metrics. We're actively expanding our data views…" | ❌ dropped. It is a note about the product's roadmap, not about this realm. What it was hedging — that only one series is measured — is now stated where it matters, in the section descriptions that name Compass as the source |
| A16 | A zero total prints "No significant change or no data available to determine trend" | ⚠️ reworded to `nothing recorded` as the metric hint. The original conflates two different findings — a measured zero and an absent measurement — which is exactly the distinction the compass gate (A8) now makes at page level |
| A17 | A non-zero total prints "`<n>` `<label>` in the last `<n>` days" | ✅ as the metric hint, `over the last 7 days` |
| A18 | Sign-ups are green, logins blue | ⚠️ the kit fixes the palette: `ActivityChart` draws logins in `#009764` and failures in `#dc2626`, and the sign-ups series takes the kit's `violet` tone. Hard-coding `oklch()` values in a page is what `style-tokens.ts` exists to prevent |
| A19 | The combined chart plots sign-ups **and** logins on one axis | ❌ **not possible with the kit.** `ActivityChart` has two fixed series, `logins` and `login_failures`; it takes no series list. The page plots logins and failures through it, and the sign-ups series through the kit's `Sparkline` in its own section, with the window total in the header. The line that would restore one combined chart is reported |
| A20 | A legend under the chart names the two series | ✅ moved into the section header, as the IAM overview and Sea Watch do (S45) |
| A21 | Y axis hides decimals, X axis has no tick line | ✅ `ActivityChart` does both |

## `feature/page-logs-events-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| A22 | When the API errors **and** returns nothing, five hard-coded events are shown instead | ❌ **deliberately dropped**, and reported. Identical defect to S1: named actors (`john.doe@company.com`), plausible IP addresses and user agents, written into a journal the domain hash-chains (`libs/ferriskey-seawatch/src/hashing.rs`) precisely so it cannot be tampered with. An empty realm now renders the empty state |
| A23 | An amber `Showing sample data` badge marks the mocked state | ❌ dropped with A22 — with no mock path there is nothing for it to mark |
| A24 | The query sends no window, no limit, no ordering | ❌ dropped, as in S2b. The server then applies `DEFAULT_LIMIT = 100` over the realm's whole history and every figure is computed over whatever those 100 rows are. The new feature pins `from_timestamp` / `to_timestamp` over 7 days with `limit: 500`, and every figure on screen names that window |

## `ui/page-logs-events.tsx`

| # | Rule | Carried over |
|---|---|---|
| A25 | Four counters: total events, successes, failures, unique actors | ✅ `MetricsBand`, with `Success rate` in place of `Successes` — the count survives as its hint — and a measured per-day sparkline behind each (S44) |
| A26 | Unique actors counts `actor_id` filtered on truthiness | ⚠️ counted as `actor_id ?? 'unknown'`, matching S5: an unattributed event is one actor rather than none |
| A27 | Status filter with three positions: All / Success / Failures | ⚠️ All / Failures plus three server-side families (Authentication, Credentials, Administration), as S10. `Success` as a filter is `All` minus `Failures` on a screen whose failure count is already in the band |
| A28 | Free-text search over `event_type`, `actor_id`, `target_id`, `target_type`, `resource`, `ip_address`, `user_agent`, `status` | ✅ same eight fields |
| A29 | Sort selector: `Most recent` / `Oldest first` | ⚠️ the feature sorts most-recent-first and every `DataView` column is sortable, `When` included — one click reverses it. A dedicated control that duplicates a column header is FK-37's inert control in another form |
| A30 | Filter and search compose | ✅ |
| A31 | Rows expand to a detail panel: event id, timestamp, target, target type, and every `details` entry as `Humanised key → <code>value</code>` | ⚠️ partially. `DataView` rows do not expand. `details.reason` and `details.error_code` are hoisted into the `Outcome` cell (S23), the remaining entries are summarised in the card-view footer (S24), and the event id has no home — reported below |
| A32 | The event type is humanised for display | ✅ through `eventLabels` in `next/pages/iam/seawatch/event-catalogue.ts` rather than by title-casing the raw string (S16) |
| A33 | A failure row is visually distinct: red tile, red `XCircle`, red pill | ✅ `IconTile danger` + `Pill danger` — form and colour, not colour alone |
| A34 | The actor label falls back `actor_id → target_id → 'Unknown'` | ✅ `actorLabel`, rendering `unattributed` when both are absent, and resolving the identifier to a username through `useRealmDirectory` |
| A35 | `resource` is rendered under the event title as `on <resource>` | ✅ folded into the `Target` column with `target_id` |
| A36 | `ip_address` and `user_agent` are rendered only when present | ✅ columns, `not recorded` when absent |
| A37 | The device icon is a phone when the user agent mentions iphone / android / mobile | ❌ dropped, as S22: a three-substring test over a raw header is a guess. The user agent is shown verbatim and is searchable |
| A38 | A footer line reads `Showing <n> of <m>` and `<x> success · <y> failure(s)` | ✅ `DataView` aggregates plus the count line under it |
| A39 | `isError` renders an amber `Live feed unavailable` banner saying cached data is displayed | ⚠️ tone `error`, and the second half of the sentence is gone: nothing is cached, and with A22 removed there is nothing to fall back on |
| A40 | Empty renders `No events match your filters` / `Try a different status or clear your search` | ✅ and the page now distinguishes "the realm recorded nothing" from "the filter hides everything" (FK-13), which the old screen did not |

## New behaviour

| # | Rule | Source |
|---|---|---|
| A41 | No screen in this section carries a create action, and every empty state says what will populate the list. These are journals: an administrator does not fabricate an execution, an event or a delivery | FK-15 |
| A42 | `Sessions` states in one line that the API exposes no realm-wide listing of active sessions, and shows what *is* measured realm-wide: the `session_created` / `session_revoked` journal. It offers no revocation control, because a security event does not carry the session id that `DELETE /users/{user_id}/sessions/{session_id}` requires | FK-37, mission |
| A43 | `Message delivery` shows the email journal — `email_sent` / `email_not_sent` — with the recipient resolved through the directory, the `email_type` and the `error_code` of a failure, and a webhook section that lists the configured endpoints and the last time each fired, saying plainly that no per-delivery webhook log exists | FK-37, mission |
| A44 | `Message delivery` raises a warning when no SMTP configuration can be read: with none, no transactional email can leave the realm, which explains an empty journal | FK-14 |
| A45 | Every metric carries a sparkline bucketed per day over the stated window, including a flat line at zero, and none at all when the window was truncated at the limit | S44, S44a |
| A46 | `Live` lists the most recent authentication flows under the charts — the only realm-wide feed of logins as they happen. It is gated on the same Compass setting as the charts, and nothing polls: the section names the window rather than claiming to stream | mission, FK-37 |

## What was reused from the IAM console, and what differs

Reused verbatim, by import:

- `@/next/pages/iam/seawatch/event-catalogue` — `eventLabels`, `eventLabel`,
  `eventFamilies`, `eventReason`, `eventDetailSummary`, `actorLabel`, and the
  `formatRelative` / `formatTimestamp` it re-exports from
  `@/next/shared/format-date`.
- `@/next/shared/use-realm-directory` — actor and target names.
- `@/components/kit` — `MetricsBand`, `Section`, `DataView`, `Pill`, `IconTile`,
  `Button`, `EmptyState`, `Segmented`, `ActivityChart`, `Sparkline`,
  `useListingQuery`.
- The 7-day / 500-event window and its query shape, copied from
  `next/pages/iam/seawatch/feature/page-security-events-feature.tsx`.

Written here rather than reused:

- The page bodies. `next/pages/iam/seawatch/ui/page-security-events.tsx` is one
  component with a hard-coded `Sea Watch` heading and an IAM description, and it
  hard-codes its five filters. The CIAM screens are three different readings of
  the same feed — all events, session events, email deliveries — under three
  different headings. The line that would let all four share one component is
  reported below.
- The columns, in `ui/event-journal.tsx`: the same cells as Sea Watch's, but
  built by a factory so each screen picks the ones it needs — the session
  journal has no `Target` column, the email journal shows a recipient and a
  template instead of an actor.
- The alert list markup, in `ui/activity-notices.tsx`. Sea Watch inlines it, and
  the kit exports the `ListingAlert` *type* but no component that renders one
  (`front/src/components/kit/ListingPage.tsx` renders them privately).

## Divergences UI ↔ domain — reported, not fixed

**1. The console displayed fabricated security events.**

`front/src/pages/activity/feature/page-logs-events-feature.tsx:9-71` holds five
hard-coded `SecurityEvent`s and substitutes them whenever the API errors and
returns nothing, behind a `Showing sample data` badge that is easy to miss. This
is the same defect that was found and removed in the admin console's Sea Watch
(commit `e442b833`, rule S1). It is still present in the current CIAM console.
The `/next` screens never do this.

**2. No endpoint lists the active sessions of a realm.**

`GET /realms/{realm_name}/users/{user_id}/sessions` exists and returns
`UserSessionDto { id, user_id, realm_id, user_agent, ip_address, created_at,
expires_at, last_seen_at }`
(`libs/ferriskey-api-user/src/handlers/list_user_sessions.rs:19-33`), and
`DELETE …/sessions/{session_id}` revokes one. Both are addressed by user id, so
a realm-wide "active devices" view would mean one request per account. The
sub-item promises `Active devices`; what the API backs is *per-account* active
devices, plus a realm-wide journal of session openings and revocations from
Sea Watch (`SecurityEventType::SessionCreated` / `SessionRevoked`, emitted at
`core/src/domain/authentication/services.rs:1459` and `:1510`). The screen shows
the journal and names the gap. What is missing on the API side is a
`GET /realms/{realm_name}/sessions`.

**3. No endpoint returns webhook deliveries.**

`Webhook` carries `triggered_at` — the last time the endpoint fired — and
nothing else about delivery: no status, no response code, no attempt count, no
per-event row (`front/src/api/api.client.ts:470-479`). So the section can show
which endpoints exist, what they subscribe to and when each last fired, and no
more. Email delivery, by contrast, *is* journalled, because the mailer writes
`EmailSent` / `EmailNotSent` security events
(`core/src/domain/trident/services.rs:1551`, `:1571`, `:1868`, `:1888`). The two
halves of "Message delivery" are therefore backed very unevenly, and the screen
says so rather than drawing a webhook table with empty status cells.

**4. A missing SMTP configuration is indistinguishable from a forbidden one.**

`GET /realms/{realm_name}/smtp-config` answers `404` when the realm has none and
`403` when the caller may not read it (`front/src/api/api.client.ts:3751-3756`),
and `useGetSmtpConfig` collapses both into `isError`. The page therefore words
its warning as "no SMTP configuration could be read", not "none is configured".

**5. `ActivityChart` cannot plot sign-ups.**

`DailyActivityStats` carries `signups`, `logins`, `login_failures`,
`expired_logins`, `pending_logins`, `unique_login_users`, `total_flows` and
`avg_login_duration_ms`, but `ActivityChart` hard-codes two `<Area>` elements on
`logins` and `login_failures` (`front/src/components/kit/charts.tsx:124-141`).
The one screen in the product whose subject is sign-ups therefore cannot draw
them through the kit's chart.

**6. `email_sent` details carry a `user_id`, never an address.**

The mailer records `{ template_id, email_type, user_id }` and, on failure,
`{ reason, error_code, template_id, email_type, user_id }`
(`core/src/domain/trident/services.rs:1551-1587`). So the delivery journal can
name the account but not the address the message actually went to — and the
address is what one checks against a bounce. `SecurityEvent` also has no
`resource` set on these events, so nothing on the wire carries the recipient.

**7. Nothing on these screens is live.**

The section is called Activity and its first sub-item is `Live`, but no query
here polls and no endpoint streams. Every figure is a snapshot taken when the
page loaded. The screens name their window instead of implying a stream, and no
`Pause` / `Resume` control is drawn (same reasoning as Sea Watch's).

## Lines needed in read-only files — reported, not written

1. `front/src/api/sea-watch.api.ts` — the same widening rule S1b asks for. Until
   it lands, these features call `window.tanstackApi.get` directly, exactly as
   the IAM Sea Watch feature does:

   ```ts
   export const useGetSecurityEvents = ({
     realm,
     fromTimestamp,
     toTimestamp,
     limit,
     eventTypes,
   }: BaseQuery & {
     fromTimestamp?: string
     toTimestamp?: string
     limit?: number
     eventTypes?: string
   }) =>
   ```

   with `query: { from_timestamp: fromTimestamp, to_timestamp: toTimestamp, limit, event_types: eventTypes }`.

2. `front/src/components/kit/charts.tsx` — to let one chart carry both series of
   the `Live` screen, `ActivityChart` would take the keys to plot:

   ```ts
   export interface ActivityChartProps {
     data: DailyActivityStats[]
     height?: number
     series?: { key: 'logins' | 'login_failures' | 'signups'; name: string; tone: ChartTone }[]
   }
   ```

   defaulting to today's two areas, so every existing call site keeps its
   rendering.

3. `front/src/next/pages/iam/seawatch/ui/page-security-events.tsx` — two optional
   props would let the CIAM `Logs & events` screen mount the IAM component
   instead of a near-copy of it:

   ```ts
   title?: string
   description?: string
   ```

   defaulting to `'Sea Watch'` and the sentence it hard-codes at line 489.

## Not built, deliberately

- **Session revocation.** The control exists in the API
  (`DELETE /realms/{realm_name}/users/{user_id}/sessions/{session_id}`) but the
  journal rows the screen shows are security events, which carry no session id.
  A revoke button there could not be wired to anything.
- **A webhook delivery table.** See divergence 3.
- **Pagination.** Every list on these screens is one window of at most 500 rows,
  and the endpoints' `offset` is not exercised. When a realm exceeds the cap the
  screen says so in an alert (S43) rather than paging through a journal whose
  figures would then cover a different set of rows than the band above it.
