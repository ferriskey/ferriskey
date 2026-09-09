# Business rules inventory — `webhooks`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

Source screens, all under `front/src/pages/realm/` because webhooks were a tab
of realm settings: `ui/page-realm-settings-webhooks.tsx`,
`ui/page-realm-settings-create-webhook.tsx`,
`ui/page-realm-settings-edit-webhook.tsx`,
`components/manage-webhook-headers.tsx`, `columns/list-webhooks.column.tsx`,
`validators.ts`, and the matching `feature/` files.

In `/next`, webhooks are their own section: `…/next/webhooks` (FK-20).

## `ui/page-realm-settings-webhooks.tsx` — listing

| # | Rule | Carried over |
|---|---|---|
| W1 | Search covers `name` and `endpoint` only | ✅ `searchIn` |
| W2 | The title carries the count: `Webhooks (n)` | ✅ the count moves to the `Total` metric and the table aggregate |
| W3 | Empty state: `No webhooks configured.` | ✅ plus a hint saying what a webhook is (FK-32) |
| W4 | Primary action `New Webhook` → `create` | ✅ |
| W5 | A row shows name, endpoint (mono), and an avatar | ✅ columns + card |
| W6 | Clicking a row opens the webhook's edit screen | ✅ opens `…/{id}/settings` |
| W7 | A pencil icon opens the same edit screen | ❌ dropped — the whole row is the link, a second control to the same place is redundant |
| W8 | A trash icon deletes, behind `ConfirmDeleteAlert`, with the description `Are you sure you want to delete "<name>"? This action cannot be undone.` | ⚠ moved to the detail's `DangerZone`, same confirmation and the same name quoted. `ListingPage` has no row actions; deleting from the detail is the pilot's shape (`role`). Reported. |
| W9 | Deletion toasts `Webhook deleted successfully` | ✅ |
| W10 | Every row shows a green **Enabled** badge | ❌ dropped — the badge is hard-coded and the domain has no such field. See divergences. |

## `columns/list-webhooks.column.tsx` — dead file

Declares four columns (URL, Name, Status, Last Triggered At). Nothing imports
it: the listing renders `renderRow` itself. Its `Last Triggered At` column —
`triggered_at` formatted, or the literal `Never` — is the only rule worth
keeping, and it is carried over as the listing's `Last triggered` column. Its
`Status` column hard-codes `Active`, same defect as W10.

## `ui/page-realm-settings-create-webhook.tsx` + its feature

| # | Rule | Carried over |
|---|---|---|
| C1 | Three general fields in order: name, endpoint URL, description | ✅ |
| C2 | `name` is required (`z.string().min(1)`) | ✅ `createWebhookValidator` reused verbatim from `@/pages/realm/validators` |
| C3 | `endpoint` must be a valid URL **or** the empty string, and is optional | ✅ reused verbatim — see divergences, the API requires it |
| C4 | `description` is optional | ✅ |
| C5 | Help texts: "A descriptive name for this webhook." / "The HTTPS URL that will receive events." / "Optional description for this webhook." | ✅ verbatim |
| C6 | Headers are key/value pairs, added one at a time; **both** key and value are required before the pair can be added | ✅ same rule in the new headers editor |
| C7 | Existing pairs can be edited in place and deleted individually | ✅ |
| C8 | Empty state: "No headers configured. Add headers to send custom HTTP headers with your webhook requests." | ✅ |
| C9 | Headers are flattened to `Record<string, string>` before sending | ✅ verbatim |
| C10 | Triggers are grouped into six categories from `@/utils/webhook-utils` | ✅ same catalogue, same six categories, reused verbatim |
| C11 | A trigger toggles on click; the selection is the `subscribers` payload | ✅ |
| C12 | The create bar appears only when the form is valid | ✅ |
| C13 | Cancel returns to the webhooks listing | ✅ |
| C14 | On success: navigate to the listing and toast `Webhook created successfully` | ✅ |
| C15 | On error, a `{ field, message }[]` error body maps each message onto its field; otherwise `error.message` is toasted, falling back to `Failed to create webhook` | ✅ verbatim, into a server-error record instead of `form.setError` |
| C16 | `console.log(selectedTriggers)` on every change | ❌ dropped, debug leftover |

## `ui/page-realm-settings-edit-webhook.tsx` + its feature

| # | Rule | Carried over |
|---|---|---|
| E1 | Same fields, same help texts, same trigger picker as create | ✅ |
| E2 | The form is prefilled from the webhook: `name ?? ''`, `description ?? ''`, `endpoint`, and `subscribers.map(s => s.name)` | ✅ |
| E3 | **Headers are never prefilled**: the API marks them `skip_serializing`, so there is nothing to show | ✅ kept, and now said on screen rather than only in a code comment |
| E4 | Headers are sent **only** when the user typed at least one pair with a non-blank key. An empty list means "untouched", not "delete" | ✅ predicate copied verbatim — this is the rule that stops an endpoint edit from wiping the headers that authenticate deliveries |
| E5 | The save bar appears only when the form is both valid **and** dirty | ✅ dirty count + validity |
| E6 | On success: navigate to the listing and toast `Webhook updated successfully` | ✅ verbatim — and it is load-bearing: `useUpdateWebhook` invalidates no query, so staying on the detail would leave it showing the pre-save response |
| E7 | Same field-error mapping as C15, with `Failed to update webhook` | ✅ |

## New in `/next`, from the prototype

- Detail tabs `Settings` / `Events` / `Deliveries` as URL segments (FK-16).
- The trigger picker is a single page of six labelled groups with a
  cross-category search, per-group `n/total` counters and per-group
  all/none actions, replacing the two-pane `Tabs` + `ScrollArea` that showed
  one category at a time and hid the other five.
- A `Signature` section stating that the delivery secret exists and is never
  returned (FK-34, partially — see divergences).

## Divergences UI ↔ domain — reported, not fixed

1. **A webhook has no "active" flag, yet the UI asserts one.**

   - Domain: `libs/ferriskey-webhook/src/entities/webhook.rs` defines
     `Webhook { id, endpoint, headers, secret, name, description, subscribers,
     triggered_at, updated_at, created_at }`. There is no `enabled`, `active`
     or `status` field, and `Webhook::new` takes none. The migrations agree:
     `core/migrations/20250711222040_create_webhooks.up.sql` and the later
     `20260819150000_harden_webhook_delivery.up.sql` add `secret`,
     `last_delivery_status` and `last_delivery_error` — never an enabled column.
   - Generated types: `Schemas.Webhook` in `front/src/api/api.client.ts:470`
     lists exactly `created_at, description, endpoint, id, name, subscribers,
     triggered_at, updated_at`.
   - Current UI: `ui/page-realm-settings-webhooks.tsx` renders a green
     **Enabled** pill on *every* row, unconditionally — it is a literal in the
     JSX, bound to nothing. `columns/list-webhooks.column.tsx` does the same
     with a green **Active** badge.
   - Prototype: `WebhooksPage.tsx` says so explicitly in a comment on the
     `triggeredAt` column — "the domain has no active flag; what says a webhook
     is alive is its last trigger date" — and drops the badge.

   `/next` shows no enabled/disabled badge. What it shows instead is real:
   `triggered_at`, and whether the webhook has any subscriber at all.

2. **The prototype's delivery health is entirely fictional.** `health`,
   `healthDetail`, `failures24h`, `deliveries24h`, the delivery history, the
   per-attempt timeline, the signature headers and the replay action come from
   `../ferriskey-kit/src/mocks/`. The API exposes five webhook routes only
   (`libs/ferriskey-api-webhook/src/router.rs`): list, get, create, update,
   delete. There is **no** deliveries endpoint. The columns `Livraison` and
   `Échecs 24 h`, the metrics `Livraisons 24 h` / `Échecs 24 h`, and
   `webhook-detail/DeliveryList.tsx` are therefore not reproduced; the
   `Deliveries` tab states that the API does not expose delivery history.
   **FK-31 (a failed delivery shows its failing step and error code on the row)
   cannot be implemented**: the data does not leave the database.

   Note that the data partly *exists*: `webhooks.last_delivery_status` and
   `webhooks.last_delivery_error` are columns added by
   `20260819150000_harden_webhook_delivery.up.sql`, but the `Webhook` domain
   struct does not carry them, so they never reach the API.

3. **The signing secret can neither be read nor rotated (FK-34).**
   `Webhook::secret` is `#[serde(skip_serializing)]` and minted once in
   `Webhook::new` — the entity's own test asserts the serialized response never
   contains it. There is no rotate/regenerate route and no `secret_hint` field,
   so the prototype's masked hint and its "Régénérer" button have nothing
   behind them. `/next` states the rule and shows the value as masked; it does
   **not** render a rotate button that would do nothing.

4. **No test-send route.** The prototype's "Envoyer un événement de test" has
   no endpoint. Dropped.

5. **`endpoint` is optional in the front validator and required by the API.**
   `createWebhookValidator` accepts `undefined` and `''`
   (`front/src/pages/realm/validators.ts`), while
   `libs/ferriskey-webhook/src/entities/webhook.rs` has `endpoint: String` and
   the create handler requires it. A webhook created with an empty endpoint is
   refused by the server, not by the form. The validator is reused verbatim
   rather than tightened — changing what the form accepts is a behaviour
   change, not a style migration.

6. **`WebhookTrigger` has 40 variants; the console's catalogue groups 38.**
   `front/src/utils/webhook-utils.ts` `WEBHOOK_CATEGORIES` omits
   `role.deleted` and `role.permission.updated`, although
   `WEBHOOK_TRIGGER_LABELS` does label them. Those two triggers cannot be
   subscribed to from the console. The catalogue is reused verbatim; the
   counters in the new picker say `n/38`, which is the number the picker can
   actually reach.

## Deviations from the mission's reading of FK-28

The mission asks for `EntityPicker` for a webhook's subscribers and
`ChipInput` for its headers. Neither is used, for reasons of kind:

- **Subscribers are enum variants, not entities.** They are 38 values of
  `WebhookTrigger`, fixed at compile time, with a category and a human label —
  not rows the administrator created. `EntityPicker` renders the selection as a
  list and everything else behind a popover, which loses the category grouping
  and the coverage counters the prototype introduced precisely to fix the
  current two-pane picker. The prototype's grouped grid
  (`webhook-detail/SubscribersField.tsx`) is reproduced instead.
  `EntityPicker` **is** used, per FK-28, where the field really does point at
  existing records: the realm maintenance whitelist (`rules/realm.md`, M4).
- **A header is a pair, not a chip.** `ChipInput` holds `string[]`; the payload
  is `Record<string, string>`. A chip field cannot express a value, and C6
  requires both halves before a header can be added.
