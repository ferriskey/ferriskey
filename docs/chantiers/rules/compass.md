# Business rules inventory — `compass`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Sources: `front/src/pages/compass/ui/page-flows.tsx`, `ui/stats-cards.tsx`,
`ui/flow-list.tsx`, `ui/page-flow-detail.tsx`, `ui/flow-timeline.tsx`, plus
the two features under `front/src/pages/compass/feature/`.

## `ui/page-flows.tsx` + `ui/stats-cards.tsx` + `ui/flow-list.tsx`

| # | Rule | Carried over |
|---|---|---|
| C1 | Four statistics come from `GET /compass/v1/stats`, never from the loaded rows: `total`, `success_count`, `failure_count`, `avg_duration_ms` | ✅ `MetricsBand` — the rows are one page, the stats cover the realm |
| C2 | `avg_duration_ms` is rounded before display; absent → `—` | ✅ `formatDuration(Math.round(...))`, absent → `—` |
| C3 | Status filter with four positions: All, Success, Failure, Pending | ⚠️ replaced by `ListingPage` filters `Failed` / `Unfinished` / `No user` (FK-13 idiom, and the prototype's set). `Success` and `Pending` are reachable by sorting the `Outcome` column; `Expired` — absent from the old filter row — is now visible in an alert |
| C4 | Free-text search covers `grant_type`, `client_id`, `user_id`, `ip_address`, `user_agent` | ✅ `searchIn`, plus `id` and `status` (prototype) |
| C5 | Filters and search compose (both applied) | ✅ `ListingPage` composes them |
| C6 | The section title carries the count of **filtered** flows | ✅ `ListingPage` footer `n of m` |
| C7 | `isError` renders a destructive alert `Flows unavailable` / "We couldn't fetch the latest flows. Please try again later." | ✅ carried as a `ListingPage` alert, tone `error` |
| C8 | Loading renders skeleton rows, not an empty state | ✅ `DataView loading` |
| C9 | Empty list renders a dedicated empty state, no create action | ✅ FK-15 — no `emptyAction` |
| C10 | A row is clickable and opens the flow detail | ✅ `getHref` |
| C11 | The row shows the grant type humanised (`authorization_code` → `Authorization Code`) | ⚠️ deliberately dropped. `grant_type` is a protocol value, not a human label: FK-04 puts it in a mono `Pill` verbatim. Title-casing it invents a name the domain never uses |
| C12 | The row shows `duration_ms` as `<n>ms` when present, nothing when absent | ⚠️ carried with FK-33 units (`184 ms` / `28.9 s`); absent → `—`, named rather than blank (FK-32) |
| C13 | The row shows `client: <client_id>` only when a client is set | ✅ dedicated `Client` column, absent → `unresolved` (FK-32) |
| C14 | The row shows `started_at` formatted with the browser locale, `Invalid date` on an unparseable value | ✅ same formatter, same guard |
| C15 | Status badge has four distinct renderings: success / failure / pending / expired | ✅ `Pill` tones success / danger / info / amber (FK-02, FK-30) |
| C16 | The grant-type avatar colours four known grants and falls back to grey + first letter | ⚠️ dropped. FK-01: a hard-coded per-grant palette makes a classification look like a state. The card avatar now encodes *user identified / anonymous*, which is a real state |
| C17 | Realm name shown as a badge next to the page title | ⚠️ dropped, as in the `role` pilot: the `/next` shell breadcrumb already names the realm |

## `ui/page-flow-detail.tsx`

| # | Rule | Carried over |
|---|---|---|
| C18 | Loading renders a skeleton, not an empty state | ✅ |
| C19 | `isError || !flow` renders a dedicated "flow not found" screen | ✅ panel, same wording intent |
| C20 | Back navigates to the flow listing | ✅ |
| C21 | Header shows grant type, status badge, and duration badge when present | ✅ |
| C22 | The flow id is shown in monospace under the title | ✅ FK-04 |
| C23 | Metadata cards are conditional: `Started` always; `Completed`, `Client`, `User`, `IP address` only when set | ⚠️ inverted. FK-32: the four facts are always shown and the absence is *named* — `never` / `in progress`, `unresolved`, `never identified`. A card that disappears cannot be distinguished from a card that was never rendered |
| C24 | Timestamps use the browser locale, `Invalid date` on an unparseable value | ✅ |
| C25 | Steps are sorted by `started_at` ascending | ✅ |
| C26 | A step with no steps recorded shows a dedicated empty message | ✅ `Section` description says so |
| C27 | Each step shows its name humanised, its status, its duration, its start time | ✅ plus the technical `step_name` in mono (FK-35) |
| C28 | `error_code` is shown in red monospace on the step | ✅ and hoisted to a page-level alert (FK-31) |
| C29 | `error_message` is shown only when `error_code` is absent, parsed as `k=v&k=v` pairs | ⚠️ dropped. The domain writes `error_message` as free text (`libs/ferriskey-compass/src/entities.rs`), never as a query string; both fields are now shown together, message as prose |
| C30 | The step graph is a `@xyflow/react` canvas with pan/zoom and computed handles | ⚠️ replaced by a vertical timeline. Flow steps are strictly sequential in the domain (`FlowRecorder::record_step` appends), so a branchable canvas models a branching that cannot occur, and it leaves no room for the error — the one thing one opens a failed flow to read |

## New behaviour, from the prototype

| # | Rule | Source |
|---|---|---|
| C31 | The failing step and its error code appear **in the table row**, under the status pill | FK-31 |
| C32 | Durations switch unit at one second | FK-33 |
| C33 | Absent client / user / completion are named, never blank | FK-32 |
| C34 | The listing has no create action, in either empty state | FK-15 |
| C35 | Failed and expired executions are announced in the alert band above the table | FK-14 |
| C36 | Metrics are four counts, no sparkline: these executions are a journal, not a measured series | FK-12 |
| C37 | An expired flow carries a dedicated explanation on its detail page: nothing failed, the user simply never came back, so no end date and no duration exist | prototype |
| C38 | The cumulated step time is stated as such, because it can be lower than the flow duration — the user's own waiting is counted nowhere | prototype |

## Deliberate divergences from the prototype

- `username`, `stepLabels` keyed on a fictional enum and `CompassFlow.username`
  do not exist in the domain. `CompassFlow` carries `user_id: Option<Uuid>`
  only (`libs/ferriskey-compass/src/entities.rs:161`). The user column shows the
  identifier in monospace, or `never identified`.
- The prototype emits one alert per failed execution. With the API's default
  page of 50 flows that is a wall of alerts above the table; a single aggregate
  line is emitted instead, naming the dominant failing step.
- The prototype's `median duration` is computed over the loaded rows. The API
  already returns `avg_duration_ms` over the whole realm; the real aggregate is
  used rather than a page-local approximation.
- `Column.secondary`, used by the prototype, does not exist in the ported
  `DataView`. Dropped.

## Divergences UI ↔ domain — reported, not fixed

**1. The listing titles each execution with its grant type, which reads as a
catalogue of flow types.**

The domain is unambiguous: `CompassFlow` is an *execution*. It is created by
`FlowRecorder::start_flow` with `status: Pending`, `started_at: Utc::now()`,
an empty `steps` vector, and is later closed by `FlowCompleted { completed_at,
duration_ms, user_id }` (`libs/ferriskey-compass/src/recorder.rs:8-20`,
`src/entities.rs:173-195`). One row is one authentication attempt.

The current console (`front/src/pages/compass/ui/flow-list.tsx:141`) makes
`formatSnakeCaseToTitleCase(flow.grant_type)` the row's title, next to a
coloured avatar bearing the grant's initial — the visual grammar of a
catalogue entry, repeated identically on every row of the same grant. The
execution's own identity (`flow.id`) appears nowhere in the list. The
prototype's reading is correct; the entity was never wrong, only its
presentation. The new view titles the row by *when it ran* and shows the grant
type as a mono classification pill.

**2. `FlowStats` has no `expired` count, though `FlowStatus` has an
`Expired` variant.**

`libs/ferriskey-compass/src/value_objects.rs:38-44` returns `total`,
`success_count`, `failure_count`, `pending_count` — but
`libs/ferriskey-compass/src/entities.rs:62-73` declares four statuses. An
expired execution is therefore counted in `total` and in nothing else, and the
realm-wide count of abandoned flows cannot be obtained from the API. The view
counts expired flows over the loaded page only, and says so.

**3. The daily activity endpoint is unused by Compass.**

`GET /compass/v1/activity/daily` returns a genuine measured series (30 days by
default: `total_flows`, `logins`, `login_failures`, `unique_login_users` —
`libs/ferriskey-api-compass/src/handlers/get_daily_activity_stats.rs:73-80`).
Only `front/src/pages/activity/` consumes it. It is the one honest source for
sparklines in this domain, but the prototype deliberately keeps Compass's
metrics as four counts, so no chart is introduced here.
