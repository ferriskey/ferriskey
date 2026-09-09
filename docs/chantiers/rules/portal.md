# Business rules inventory — `portal`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

The domain is spread over four trees in the current console:
`pages/portal/` (routing + themes list + theme builder),
`pages/portal/themes/components/` (the page builder machinery),
`pages/portal-theme/` (the legacy single-theme editor, its context, its
panels and its preview) and `pages/portal-layouts/` (layouts list + layout
builder). Themes are also mounted by the CIAM product through
`pages/console-branding/`, which is why nothing under `pages/` was touched.

## `pages/portal/page-portal.tsx` — routing

| # | Rule | Carried over |
|---|---|---|
| P1 | Every builder tab is addressable: `/themes/:id`, `/themes/:id/:section`, `/themes/:id/pages/:page_type` | ✅ FK-16 — `useRouteTabs` for the detail sections, a dedicated route for the page builder |
| P2 | `/layouts/:layout_id` opens the layout builder; the literal `new` means "unsaved" | ✅ |
| P3 | The list pages render their own chrome (Themes \| Layouts tabs) | ✅ `ui/portal-page-header.tsx` |
| P4 | `/theme` — legacy single-theme-per-realm editor, kept "until cleanup PR" | ❌ **not carried over.** It edits `realm.portal_theme`, the pre-collection resource that `/themes` replaced; the current console keeps it only as a fallback until its cleanup PR. Migrating it would give `/next` two editors for two shapes of the same thing. |
| P5 | `/builder-demo` — sandbox | ❌ **not carried over.** A developer sandbox, not a console screen. |

## `pages/portal/components/portal-overview-header.tsx`

| # | Rule | Carried over |
|---|---|---|
| P6 | Title `Portal Customization`, description mentioning themes and layouts | ✅ shortened to `Portal` + "Appearance of the public authentication pages of this realm." (FK-36 — the old description repeated the tab labels) |
| P7 | Two tabs, Themes and Layouts | ✅ |
| P8 | In the CIAM console the Layouts tab is dropped (layouts are admin-only) | ❌ **not applicable.** `/next` is the IAM console only; the CIAM branch of this component has no reader here. |
| P9 | Optional primary action rendered top-right | ✅ |

## `themes/ui/page-themes-list.tsx` + `themes/feature/page-themes-list-feature.tsx`

| # | Rule | Carried over |
|---|---|---|
| P10 | Heading carries the count — `Themes (n)` | ✅ as a `MetricsBand` figure and the section title |
| P11 | Import goes through a hidden `<input type=file accept="application/json,.json">`; the visible button triggers it | ✅ verbatim |
| P12 | The file input resets `value` after each change so picking the same file twice fires again | ✅ verbatim |
| P13 | Loading renders skeleton rows, not an empty state | ✅ |
| P14 | Empty state: `No themes yet. Create one to start customizing the portal.` + a create button | ✅ reworded, FK-13 |
| P15 | Row subtitle is `theme_id: <id>` | ✅ moved to the mono identifier under the row; the subtitle now names the layout and the last update, which is what the prototype shows |
| P16 | The active theme carries an `ACTIVE` badge | ✅ `Pill tone='success'` |
| P17 | Activate is offered **only** when the theme is not already active | ✅ |
| P18 | Delete is disabled on the active theme, with the title `Cannot delete the active theme` | ✅ upgraded to FK-37/FK-39: an inert control with the tooltip "The active theme cannot be deleted. Activate another one first." |
| P19 | Edit and Export are always offered | ✅ |
| P20 | Create dialog: the name is trimmed, empty is refused, `Enter` submits, the button reads `Creating…` and is disabled while the mutation runs | ✅ verbatim |
| P21 | Which theme is active comes from `GET /portal/active?page_type=login`, not from a flag on the list | ✅ verbatim |
| P22 | A new theme is created with `config: defaultTheme` | ✅ verbatim |
| P23 | A new theme's twelve pages are seeded with `defaultPageTree(pageType)` **sequentially**, never in parallel — twelve concurrent calls would each start their own token refresh with the same refresh token, and reuse detection revokes a rotated family | ✅ copied verbatim, comment included as a rule here rather than in the code |
| P24 | Failed seeds are counted and reported once: `<n> page(s) could not be pre-filled — open them before activating this theme.` | ✅ verbatim |
| P25 | After a successful create the console navigates straight into the theme | ✅ now to the theme's detail page rather than the builder — the builder is now per-page |
| P26 | Export failure toasts `Could not export this theme` | ✅ verbatim |
| P27 | Import posts the parsed envelope as-is; a parse failure toasts the reader's message | ✅ verbatim |

## `themes/ui/page-theme-builder.tsx` + `themes/feature/page-theme-builder-feature.tsx`

The single full-viewport builder is split in `/next` into a **detail page**
(Theme / Layout / Pages) and a **page builder** reached from the Pages tab.
Every rule below survives that split.

| # | Rule | Carried over |
|---|---|---|
| P28 | Twelve page types with fixed labels and a fixed order | ✅ same list, same order, in `portal-pages.ts` |
| P29 | Left nav: `Theme`, `Layout`, then a `Pages` group | ✅ becomes the three detail tabs plus the page rail inside the builder |
| P30 | The theme name is editable inline | ✅ moved to a `FieldRow` in the Identity section (FK-06 — an 18 px borderless input in the chrome is a third type scale) |
| P31 | Activate is disabled while activating and reads `Activating…` | ✅ |
| P32 | Save sends metadata (`name`, `layout_id`, `config`) plus only the pages actually edited | ✅ split: the detail page saves metadata, the page builder saves its own page |
| P33 | Save is disabled while either mutation runs | ✅ |
| P34 | Layout select: `none` maps to `null`; each option appends ` (default)` when `is_default` | ✅ |
| P35 | The help text warns that the layout change needs an explicit save | ✅ replaced by the save bar, which states it structurally |
| P36 | A layout tree is read either as an array or as `{ children: [...] }` | ✅ `parseTree`, copied verbatim |
| P37 | `theme.pages` is keyed in **camelCase** (`forgotPassword`), while `PortalPageType` is snake_case — the key must be converted or every multi-word page reads as empty | ✅ `readPageTree` / `snakeToCamel` copied verbatim. This is the single most breakable rule of the domain. |
| P38 | Theme tokens tab = live preview + the five panels (Buttons, Inputs, Widget, Typography, Page) | ✅ the product components are mounted unchanged |
| P39 | Save fires metadata + N pages with `Promise.allSettled` and emits exactly one toast, so a 422 on a tree cannot hide behind a green metadata toast | ✅ no longer needed for the detail page (one call), preserved in the page builder through `describePortalPageError` |
| P40 | Failures are described per page: `<page label> — <detail>` | ✅ `describePortalPageError` reused verbatim |
| P41 | Unknown URL section falls back to `theme` | ✅ `useRouteTabs` does it (FK-16) |
| P42 | `fillParent` when rendered inside the CIAM `ProductLayout`, viewport calc otherwise | ❌ **not applicable.** `/next` has exactly one host, the app shell; the builder fills it. |

## `themes/components/page-tree-editor.tsx` — the page builder

Kept as the product implementation. Imported, not rewritten.

| # | Rule | Carried over |
|---|---|---|
| P43 | Required blocks for the edited page come from `GET /portal/page-requirements` | ✅ component imported unchanged |
| P44 | `missing` = required types absent from the tree at **any** depth (`collectTypes` walks every value) | ✅ unchanged |
| P45 | Header badge: green `All required blocks present`, or amber `Missing: …` | ✅ unchanged |
| P46 | Device presets: iPhone 402×874, Tablet 768×1024, Desktop 1280×800 | ✅ unchanged |
| P47 | Clicking a breakpoint tab in the config panel switches the preview device (`sm`/`md` → tablet, `lg`/`xl` → desktop) | ✅ unchanged |
| P48 | The iframe is scaled to fit but keeps its device-size internal viewport so `@media` still fires | ✅ unchanged |
| P49 | The right config rail is mounted only when a node is selected; the grid is `264px 1fr 320px` / `264px 1fr` | ✅ unchanged |
| P50 | When a layout is attached, the page tree renders inside it through the `page-content` slot, with `fillHeight={false}` | ✅ unchanged |
| P51 | The left rail is split into two scroll regions so a long block list cannot push the page nav off-screen | ✅ unchanged; the page nav passed as `leftRailNav` is the `/next` one |

## `themes/components/page-component-library.tsx` — block library filtering

The three filters named in the mission. All three come from
`lib/builder-portal/components.tsx` and are applied by the library component,
which `/next` imports unchanged — so they are carried over by construction.

| # | Rule | Carried over |
|---|---|---|
| P52 | **`LAYOUT_ONLY_BLOCK_TYPES`** — `page-content`, `card-header`, `card-content`, `card-footer`. Never offered in a *page* palette: `page-content` belongs to a layout tree, the three `card-*` slots are pre-populated by the Card's `getDefaultNode` and are never authored standalone. The layout library does the inverse: it shows exactly this set, under "Required for this layout". | ✅ unchanged |
| P53 | **`HIDDEN_BLOCKS_BY_PAGE_TYPE`** — per-page exclusion list, keyed by `PortalPageType`, applied to **both** the generic palette and the "Required for this page" group. A page absent from the table gets the full palette. Eleven of the twelve pages are listed; `login` hides only `back_to_login_link`, `device_verified` hides fourteen types. | ✅ unchanged |
| P54 | **`RESTRICTED_TO_PAGE_TYPE`** — the inverse allow-list, keyed by block type: `totp_qr_code` and `totp_secret` only on `totp_setup`; `user_code_input`, `device_consent`, `device_approve_button`, `device_deny_button` only on `device_verify`. A block with an allow-list is hidden everywhere else, and also hidden when no page type is known. | ✅ unchanged |
| P55 | `REQUIRED_BLOCK_TYPES` never appears in the generic palette — only in "Required for this page", and only when the API asked for it | ✅ unchanged |
| P56 | Groups: Layout / Content / Form & Actions / Identity fields, plus `Other` for any block that no group claims (so a newly added block surfaces instead of disappearing) | ✅ unchanged |
| P57 | Presets are click-to-insert, never draggable; inserting over a non-empty tree asks for confirmation first | ✅ unchanged |
| P58 | The Components / Presets / Tree tabs stay pinned outside the scroll region | ✅ unchanged |

## `portal-layouts/ui/page-portal-layouts-list.tsx` + its feature

| # | Rule | Carried over |
|---|---|---|
| P59 | Heading carries the count — `Layouts (n)` | ✅ |
| P60 | Import: same hidden-input mechanism, same reset | ✅ verbatim |
| P61 | Loading renders skeleton rows | ✅ |
| P62 | Empty state: `No portal layouts yet. Create one to get started.` | ✅ reworded, FK-13 |
| P63 | Row subtitle is `layout_id: <id>` | ✅ kept as the mono identifier; the subtitle now carries node count and last update |
| P64 | Edit / Export / Delete on every row | ⚠️ **changed.** Delete becomes inert with a tooltip when the domain would refuse it (FK-39, see below). |
| P65 | `New Layout` navigates to `/layouts/new` | ✅ |
| P66 | Export failure toasts `Could not export this layout` | ✅ verbatim |
| P67 | Import posts the parsed envelope as-is | ✅ verbatim |
| P68 | The list props type omits `is_default` entirely — the default layout is invisible in the current console | ⚠️ **changed.** `is_default` is displayed and drives the delete guard. See the divergences below. |

## `portal-layouts/ui/page-portal-layout-builder.tsx` + its feature

| # | Rule | Carried over |
|---|---|---|
| P69 | `new` is a sentinel: no fetch, empty tree, and the save creates then rewrites the URL with `replace: true` | ✅ verbatim |
| P70 | A new layout starts with an **empty** tree; `page-content` is offered in the library, not seeded | ✅ verbatim |
| P71 | Save is disabled while saving **and** when the name is empty; the label is `Create` when new, `Save` otherwise | ✅ verbatim |
| P72 | The canvas is themed with the realm's legacy portal theme merged over `defaultTheme` | ✅ verbatim (`useGetPortalTheme` + `mergeWithDefaults`) |
| P73 | `LayoutComponentLibrary`: generic = neither required nor layout-only; "Required for this layout" = exactly `LAYOUT_ONLY_BLOCK_TYPES` | ✅ component imported unchanged |
| P74 | The layout canvas is `padded={false}` and `maxWidth = viewportWidth` — a layout owns its own gutters | ✅ verbatim |
| P75 | The builder is keyed on the loaded layout id so switching layouts remounts the tree state | ✅ verbatim |

## Rules added by the FerrisKey style, not present in the current screens

| # | Rule | Where |
|---|---|---|
| A1 | FK-39 — `PortalThemeActive` is announced before the click: the delete control on the active theme is inert and carries its remedy | themes listing, theme detail |
| A2 | FK-39 — `PortalLayoutDefault` and `PortalLayoutInUse` are announced before the click: the delete control is inert and names either "the default layout" or the themes that hold it | layouts listing |
| A3 | FK-37 — every refusal states the remedy: "The active theme cannot be deleted. Activate another one first.", "Detach it from that theme first." | both |
| A4 | FK-41 — `validate_pages` returns every faulty page at once, so the console shows every faulty page at once, each with its missing block types, on the Pages tab and in the activation tooltip. No page-by-page discovery. | theme detail, themes listing |
| A5 | FK-38 — "layout" here is a portal layout only. The word is never used for an email template, and the layout tab says what a layout is ("header / footer / card wrapper reused by every page of the theme"). | everywhere |
| A6 | FK-16 — the three detail sections are URL segments | theme detail |
| A7 | FK-32 — a theme with no layout reads `No layout`, not an empty cell; an unused layout reads `unused` | both listings |

## Divergences UI ↔ domain — reported, not fixed

**1. The console can never change which layout is the default.**
`useSetDefaultPortalLayout` exists in `front/src/api/portal-layouts.api.ts:104`
and maps `PUT /realms/{realm_name}/portal-layouts/{layout_id}/default`, but no
screen in the current console calls it, and `PortalLayoutListItem` in
`pages/portal-layouts/ui/page-portal-layouts-list.tsx:7` does not even carry
`is_default`. Meanwhile `libs/ferriskey-portal-layouts/src/services.rs:190`
refuses to delete the default layout. An administrator therefore meets a
refusal it has no way to lift from the console. Not fixed here: adding the
action changes what the console can do, which is a behaviour change, not a
style migration. `/next` at least *shows* `is_default` and explains the
refusal.

**2. FK-42 has no reachable target in this domain.**
The two foreign keys that could unassign on delete behave differently:
`portal_themes.layout_id` is `ON DELETE RESTRICT`
(`core/migrations/20260517120000_portal_themes_per_page_trees.up.sql:23`) —
which is why the domain raises `PortalLayoutInUse` rather than unassigning —
and `realm_settings.portal_theme_id` is `ON DELETE SET NULL` (same file,
line 33) but is guarded by `CoreError::PortalThemeActive`
(`libs/ferriskey-portal-theme/src/services.rs:310`) before the database ever
gets the chance. So no portal deletion silently unassigns anything, and the
amber "this will detach…" wording FK-42 asks for has nothing to attach to.
The nearest real case is the *opposite* direction — changing a theme's layout
detaches the previous one — which the layout tab states in amber.

**3. The activation validator and the page requirements endpoint are two
sources for the same list.**
`GET /portal/page-requirements` feeds the builder's "Required for this page"
group, while activation validates against `REQUIRED_BLOCKS` in
`libs/ferriskey-portal-theme/src/validation.rs:9`. They are the same table
today (the endpoint is built from it), but the console computes the
per-theme validity shown on the listing and on the Pages tab from the
endpoint — so if the two ever drift, the console will promise an activation
the server refuses. Reported, not fixed: collapsing them is an API change.

**4. `EmailVerified` and `DeviceVerified` require no block at all.**
`validation.rs:40` and `validation.rs:64` declare empty requirement lists on
purpose — a static confirmation screen is a valid page. The current console
shows those two pages with an empty "Missing:" badge that reads as "nothing
was checked". `/next` names it: `no required block`.
