# Business rules inventory — `console/branding`

Written before rewriting, checked after. A rule not carried over is a
migration bug, not a simplification.

The current section is `front/src/pages/console-branding/page-console-branding.tsx`,
21 lines, and it holds no logic of its own: it mounts admin features and
nothing else. Its only decisions are which routes exist and which features
answer them. The business rules of the screens themselves are already
inventoried in `docs/chantiers/rules/email-template.md` and
`docs/chantiers/rules/portal.md`, and are carried over by the act of mounting
the `/next` rebuilds rather than restating them.

## `page-console-branding.tsx`

| # | Rule | Carried over |
|---|---|---|
| B1 | Index redirects to `email-templates` | ✅ verbatim |
| B2 | The email-template list and the email-template **builder** are both reachable inside the console (`email-templates/:template_id/builder`) | ✅ widened to `email-templates/*`, which brings the `/next` rebuild's `templates`, `smtp`, `create/builder`, `:id/builder` and `:id` routes with it |
| B3 | Portal themes are rendered "inside the console chrome (shared with the admin portal; navigation is path-aware so it stays in the console)" — the current file says so in a comment | ⚠️ **the intent is carried over, the property is not.** See divergence 1: the `/next` portal screens are not path-aware, and the current console's are. This is the one place where the current console is ahead of `/next` |
| B4 | Four theme routes: list, detail, detail with a `:section`, and the page builder `themes/:theme_id/pages/:page_type` | ✅ the `/next` shape is list / `:theme_id` → redirect to `theme` / `:theme_id/*` detail with tabbed sections / `:theme_id/pages/:page_type`. Same four destinations, the section is a splat instead of a positional param |
| B5 | Portal **layouts** are not part of branding — the current console mounts themes only | ✅ `PagePortalLayoutsFeature` and `PagePortalLayoutBuilderFeature` are not mounted. `NextPagePortal` is therefore *not* mounted whole; its three theme features are mounted individually |
| B6 | No branding screen is cloned from the admin console | ✅ every route resolves to a module under `next/pages/iam/` |

## Rules the mounted screens carry, and that survive by construction

| # | Rule | Where it lives |
|---|---|---|
| C1 | FK-23 — an email template's type is disabled outside creation, **and** says why: it decides which variables the engine supplies | `next/pages/iam/email-template/ui/page-email-template-detail.tsx`, rule E29/N1 of `email-template.md` |
| C2 | FK-39 — `PortalThemeActive` is announced before the click: delete on the active theme is inert and carries its remedy ("The active theme cannot be deleted. Activate another one first.") | `next/pages/iam/portal/ui/page-portal-themes.tsx:243`, rule A1 of `portal.md` |
| C3 | FK-39 — `PortalLayoutDefault` / `PortalLayoutInUse` are announced before the click | `portal.md` rule A2 — **on the layouts listing, which branding does not mount** (B5). Nothing in this section can trigger either refusal, so nothing is lost |
| C4 | FK-43 — a variable cited by the MJML but not supplied for the type raises an amber banner instead of failing the send | `email-template.md` rule N3 |
| C5 | FK-40 — no "Send a test email" action, because no such endpoint exists | `email-template.md`, SMTP tab deviation |

## Rules deliberately not carried over

| # | Rule | Why |
|---|---|---|
| B3 | Path-aware navigation inside the theme screens | Cannot be carried over from inside this chantier's scope: it lives in `next/pages/iam/portal/portal-urls.ts` and `next/routes.ts`, both read-only. Reported with the exact lines instead of worked around — see divergence 1 |

## Deviations

| Screen | What differs | Why |
|---|---|---|
| Email templates | The console reaches the SMTP tab as well as the templates tab, which the current console did not | `NextPageEmailTemplates` puts both behind one splat, and SMTP is the reason half the branding features deliver anything at all; hiding it would have meant re-declaring the routes one by one to exclude it |
| Email templates | The URL grows a segment: `branding/email-templates` redirects to `branding/email-templates/templates` | The `/next` rebuild anchors its tabs in the URL (FK-16). `isSubItemActive` matches on `startsWith`, so the rail entry stays lit |
| Themes | `NextPagePortal` is decomposed rather than mounted | B5 — mounting it whole would have added a Layouts route the section's rail does not offer |
| Themes | The theme detail keeps its `Portal` page title and its Themes / Layouts tab strip | Both come from `next/pages/iam/portal/ui/portal-page-header.tsx`, read-only. Divergence 1 |

## UI ↔ domain divergences found

**1. The `/next` admin screens are anchored to the admin console's URLs, and
the console's are not.** This is not a domain divergence but it is the finding
of this mission, so it is recorded here in full.

Every `/next` screen this section mounts builds its own links from a
module-level constant rooted at `NEXT_URL(realm)`:

- `next/pages/iam/email-template/feature/page-emails-feature.tsx:21` —
  `useRouteTabs(NEXT_EMAIL_TEMPLATES_URL(realm), EMAIL_TABS)`. Mounted at
  `…/next/console/branding/email-templates/templates`, `useRouteTabs`'
  `pathname.startsWith(basePath)` is false, so the tab value falls back to
  `tabs[0]` (Templates reads as active even on SMTP) and both tab hrefs point
  at `…/next/email-templates/…` — outside the console.
- `next/pages/iam/email-template/feature/templates-tab-feature.tsx:16`,
  `page-email-template-detail-feature.tsx:29,86`,
  `page-email-template-builder-feature.tsx:20` — every create / edit / back
  navigation leaves the console.
- `next/pages/iam/portal/portal-urls.ts:3-19` — all five theme helpers derive
  from `NEXT_PORTAL_URL`, so opening a theme, opening a page builder, and
  going back all leave the console.
- `next/pages/iam/portal/ui/portal-page-header.tsx:35-36` — renders a
  Themes / Layouts strip whose Layouts entry has no console route at all.
- `next/pages/iam/identity-providers/feature/page-providers-overview-feature.tsx:34`,
  `page-provider-detail-feature.tsx:54`,
  `page-create-provider-feature.tsx:93` — same, for the authentication section.

The current console does not have this problem: it mounts
`front/src/pages/identity-providers/…` and `front/src/pages/portal/themes/…`,
whose helpers are path-aware — which is exactly what the comment at
`page-console-branding.tsx:13-14` claims. The `/next` rebuild dropped that
property when it replaced the path-aware helpers with constants.

Not worked around. The lines needed are in the report.

**2. The console's theme screens introduce themselves as "Portal".**
`portal-page-header.tsx:26` prints `Portal` with the description "Appearance of
the public authentication pages of this realm." Inside the CIAM console the
rail calls this entry Themes / "Portal appearance", so the page contradicts its
own rail entry. Consequence of mounting rather than cloning; reported, not
fixed.
