import type { CSSProperties, FormEvent, MouseEvent, ReactNode } from 'react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import {
  useGetActivePortalTheme,
  useGetPortalPageRequirements,
} from '@/api/portal-theme.api'
import { useGetPublicPortalLayout } from '@/api/portal-layouts.api'
import { useGetLoginSettings } from '@/api/realm.api'
import { useSetupOtp } from '@/api/trident.api'
import type { RouterParams } from '@/routes/router'
import type { BuilderNode } from '@/lib/builder-core'
import { generateBreakpointCss, treeToReactNode } from '@/lib/builder-portal'
import { mergeWithDefaults, themeToCssVars } from '@/pages/portal-theme/lib/theme'
import type { Schemas } from '@/api/api.client'
import { usePortalPageSubmit } from '../hooks/use-portal-page-submit'
import { usePasskeyAuth } from '../hooks/use-passkey-auth'
import {
  DeviceVerifyResult,
  DeviceVerifyShell,
} from '../ui/page-device-verify'

function parseTree(tree: unknown): BuilderNode[] {
  if (Array.isArray(tree)) {
    return tree as BuilderNode[]
  }
  if (tree && typeof tree === 'object' && Array.isArray((tree as { children?: unknown }).children)) {
    return (tree as { children: BuilderNode[] }).children
  }
  return []
}

function collectTypes(nodes: BuilderNode[], acc: Set<string>) {
  for (const node of nodes) {
    if (node && typeof node.type === 'string') acc.add(node.type)
    if (Array.isArray(node?.children)) collectTypes(node.children, acc)
  }
}

function hasAllRequiredBlocks(tree: BuilderNode[], required: string[]): boolean {
  if (required.length === 0) return true
  const present = new Set<string>()
  collectTypes(tree, present)
  return required.every((type) => present.has(type))
}

// A layout without a `page-content` slot has nowhere to render the page,
// so it would produce a blank screen — treat it the same as "no layout"
// and fall back to the default hardcoded page.
function layoutHasPageContent(tree: BuilderNode[]): boolean {
  const present = new Set<string>()
  collectTypes(tree, present)
  return present.has('page-content')
}

interface Props {
  children: ReactNode
  pageType: Schemas.PortalPageType
}

export function PortalLayoutWrapper({ children, pageType }: Props) {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { data: activeData, isLoading: isThemeLoading } = useGetActivePortalTheme({
    realm,
    pageType,
  })
  // Device-verify only: the approved-state success screen is a separate
  // themeable page (`device_verified`). Fetch its tree alongside so we can
  // swap the hardcoded result for the admin's custom design once the user
  // approves. Enabled only on the device-verify page to avoid an extra call
  // everywhere else.
  const { data: verifiedData } = useGetActivePortalTheme({
    realm,
    pageType: 'device_verified',
    enabled: pageType === 'device_verify',
  })
  const layoutId = activeData?.layout_id
  const { data: layoutData, isLoading: isLayoutLoading } = useGetPublicPortalLayout({
    realm,
    layoutId: layoutId ?? '',
  })
  const { data: requirementsData, isLoading: isRequirementsLoading } =
    useGetPortalPageRequirements({ realm })
  // Pulled separately from the auth feature so portal pages that don't render
  // the React `<PageLogin />` fallback still have the realm's configured
  // identity providers available to the `identity_providers` block.
  const { data: loginSettings } = useGetLoginSettings({ realm })
  const identityProviders = loginSettings?.identity_providers ?? []

  // TOTP setup: when the user is mid-enrolment we need to fetch the QR
  // code + secret from the backend's setup endpoint. The token lives in
  // the URL as `client_data` (set by the auth flow). Only run the query
  // on the totp_setup page — every other page would waste a network call
  // and risk a 4xx if no token is present.
  const totpSetupToken =
    pageType === 'totp_setup'
      ? new URLSearchParams(typeof window === 'undefined' ? '' : window.location.search).get(
          'client_data',
        )
      : null
  const { data: totpSetupData } = useSetupOtp({
    realm,
    token: totpSetupToken,
  })
  const totpSetup = useMemo(
    () => {
      if (
        pageType === 'totp_setup' &&
        totpSetupData?.otpauth_url &&
        totpSetupData?.secret
      ) {
        return {
          otpauthUrl: totpSetupData.otpauth_url,
          secret: totpSetupData.secret,
        }
      }
      return undefined
    },
    [pageType, totpSetupData],
  )

  // Cache the *already-computed* CSS vars in localStorage keyed by realm
  // so a refresh can apply the realm's last-known theme instantly. The
  // pre-computed shape lets us read the same cache from a tiny inline
  // script in `index.html` that runs before React mounts — that script
  // applies the vars to `:root` before the browser paints, killing the
  // last ~50ms of flash that survived the React-side fallback alone.
  const cachedVars = useMemo(() => readCachedCssVars(realm), [realm])
  const tokens = activeData?.design_tokens
  const cssVars = useMemo(() => {
    if (tokens) {
      return themeToCssVars(mergeWithDefaults(tokens)) as unknown as CSSProperties
    }
    if (cachedVars) {
      return cachedVars as unknown as CSSProperties
    }
    return themeToCssVars(mergeWithDefaults(undefined)) as CSSProperties
  }, [tokens, cachedVars])
  useEffect(() => {
    if (activeData?.design_tokens) {
      writeCachedCssVars(
        realm,
        themeToCssVars(mergeWithDefaults(activeData.design_tokens)) as Record<
          string,
          string
        >,
      )
    }
  }, [realm, activeData?.design_tokens])

  const pageTree = parseTree(activeData?.page_tree)
  const layoutTree = layoutData?.data ? parseTree(layoutData.data.tree) : []
  const requiredBlocks = useMemo(() => {
    const entry = requirementsData?.data?.find((r) => r.page_type === pageType)
    return entry?.required_blocks ?? []
  }, [requirementsData, pageType])
  // The layout only wraps the custom page when that page is valid (i.e. ships
  // all required blocks). Otherwise we fall back to the hardcoded React
  // feature, which already provides its own layout — wrapping it inside the
  // admin's layout would render an incoherent mix of two designs.
  const pageIsValid = pageTree.length > 0 && hasAllRequiredBlocks(pageTree, requiredBlocks)
  // Inline form-error banner state. Submit handlers populate this so the
  // user gets feedback on failed sign-ins / registrations / etc. without
  // having to find a corner-of-screen toast. The `form_error_banner`
  // block reads this through the render options and shows or hides
  // itself accordingly.
  const [formError, setFormError] = useState<string | null>(null)
  const { onSubmit, isSubmitting, onDeviceDeny, deviceResult, onDeviceReset } =
    usePortalPageSubmit(pageType, {
      onFormError: setFormError,
    })

  // Passkey + magic-link buttons rendered inside the custom portal tree
  // expose themselves via `data-fk-action` (see renderer.tsx). The React
  // PageLogin fallback wires real onClick handlers, but a tree authored in
  // the builder is just declarative markup — so we delegate clicks from the
  // wrapper here and route them to the same flows the fallback uses.
  const { onPasskeyLogin } = usePasskeyAuth({
    realm_name: realm,
    enabled: loginSettings?.passkey_enabled ?? false,
    isAuthInitiated: true,
  })
  const navigate = useNavigate()

  const handlePortalActionClick = useCallback(
    (event: MouseEvent<HTMLDivElement>) => {
      const trigger = (event.target as HTMLElement | null)?.closest(
        '[data-fk-action]',
      ) as HTMLElement | null
      if (!trigger) return

      const action = trigger.getAttribute('data-fk-action')
      if (action === 'passkey') {
        // The button lives inside the surrounding <form>; without preventing
        // default the click would also submit the form (because `<button>`
        // defaults to type=submit when not explicitly set — `type=button` is
        // set in the renderer but we belt-and-brace here too).
        event.preventDefault()
        onPasskeyLogin()
        return
      }

      if (action === 'device-deny') {
        // Deny lives outside the form's submit path (that's reserved for
        // approve). Read the typed user_code from the surrounding form and
        // hand it to the device-verify hook with action=deny.
        event.preventDefault()
        const form = trigger.closest('form')
        if (form && onDeviceDeny) onDeviceDeny(new FormData(form))
        return
      }

      if (action === 'magic-link') {
        event.preventDefault()
        // Carry over whatever the user already typed into the page's
        // email/username field so the dedicated form lands pre-filled — most
        // realms accept either, so try both names.
        const form = trigger.closest('form')
        const formData = form ? new FormData(form) : null
        const email = String(
          formData?.get('email') ?? formData?.get('username') ?? '',
        ).trim()
        const query = email ? `?email=${encodeURIComponent(email)}` : ''
        navigate(`/realms/${realm}/authentication/magic-link-request${query}`)
      }
    },
    [navigate, onDeviceDeny, onPasskeyLogin, realm],
  )

  if (
    isThemeLoading ||
    (layoutId && isLayoutLoading) ||
    isRequirementsLoading
  ) {
    return <div style={cssVars}>{children}</div>
  }

  // Page content: realm admin's custom tree when present, fall back to the
  // hardcoded React feature otherwise. When a custom tree is rendered, wrap
  // it in a <form> so submit_button can fire the page-specific handler.
  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!onSubmit) return
    onSubmit(new FormData(event.currentTarget))
  }

  // Collect responsive overrides from both the page tree and the layout tree
  // (each node's id is unique across them) into a single <style> block.
  const breakpointCss = [
    generateBreakpointCss(pageTree),
    generateBreakpointCss(layoutTree),
  ]
    .filter(Boolean)
    .join('\n')

  const responsiveStyle = breakpointCss ? (
    <style dangerouslySetInnerHTML={{ __html: breakpointCss }} />
  ) : null

  // Hover / active / busy CSS for portal buttons. Inline styles can't
  // express `:hover` so we inject a static rule set keyed by a marker
  // attribute (`data-fk-portal-btn`) the renderer puts on every button.
  // - hover: subtle opacity dip — works for primary, secondary, social
  //   and alt-auth buttons regardless of their underlying colour
  // - active: a touch more pronounced for the press-down feel
  // - busy: rendered `cursor: wait` + dimmer + `pointer-events: none` so
  //   the user can't double-click submit while the network call is in
  //   flight. The renderer flips `data-fk-busy` when the submit handler
  //   is in flight.
  const buttonInteractionCss = `
    [data-fk-portal-btn]:not(:disabled):not([data-fk-busy="true"]):hover {
      opacity: 0.92;
      filter: brightness(1.03);
    }
    [data-fk-portal-btn]:not(:disabled):not([data-fk-busy="true"]):active {
      opacity: 0.85;
      filter: brightness(0.97);
    }
    [data-fk-portal-btn][data-fk-busy="true"] {
      cursor: wait;
      opacity: 0.75;
      pointer-events: none;
    }
    [data-fk-portal-btn]:disabled {
      cursor: not-allowed;
      opacity: 0.55;
    }
    @keyframes fk-portal-spin {
      from { transform: rotate(0deg); }
      to { transform: rotate(360deg); }
    }
    [data-fk-portal-spinner] {
      animation: fk-portal-spin 0.8s linear infinite;
    }
  `
  const buttonInteractionStyle = (
    <style dangerouslySetInnerHTML={{ __html: buttonInteractionCss }} />
  )

  // The layout is only renderable when it has a `page-content` slot.
  // Without one, applying it would hide the page entirely.
  const layoutIsValid = layoutTree.length > 0 && layoutHasPageContent(layoutTree)

  // Device-verify terminal state: once the user has approved / denied, swap
  // the code-entry tree for the result screen. The approved state renders the
  // admin's custom `device_verified` tree when they've authored one — routed
  // through the SAME layout slot as a normal page so it sits exactly where the
  // code-entry screen does (centering comes from the active layout). Denied /
  // no custom tree falls back to the hardcoded result, which centers itself
  // via its own full-height shell.
  if (pageType === 'device_verify' && deviceResult) {
    const verifiedTree = parseTree(verifiedData?.page_tree)
    if (deviceResult === 'approved' && verifiedTree.length > 0) {
      const verifiedContent: ReactNode = treeToReactNode(verifiedTree, {
        runtime: true,
        identityProviders,
      })
      const verifiedStyleCss = [
        generateBreakpointCss(verifiedTree),
        generateBreakpointCss(layoutTree),
      ]
        .filter(Boolean)
        .join('\n')
      const verifiedStyle = verifiedStyleCss ? (
        <style dangerouslySetInnerHTML={{ __html: verifiedStyleCss }} />
      ) : null
      return (
        <div style={cssVars} onClick={handlePortalActionClick}>
          {verifiedStyle}
          {buttonInteractionStyle}
          {layoutIsValid
            ? treeToReactNode(layoutTree, {
                runtime: true,
                pageContent: verifiedContent,
                identityProviders,
              })
            : verifiedContent}
        </div>
      )
    }
    return (
      <div style={cssVars}>
        <DeviceVerifyShell>
          <DeviceVerifyResult
            status={deviceResult}
            onBackToStart={() => onDeviceReset?.()}
          />
        </DeviceVerifyShell>
      </div>
    )
  }

  if (!pageIsValid || (layoutTree.length > 0 && !layoutIsValid)) {
    return (
      <div style={cssVars}>
        {responsiveStyle}
        {buttonInteractionStyle}
        {children}
      </div>
    )
  }

  const pageContent: ReactNode = (
    <form onSubmit={handleSubmit}>
      {treeToReactNode(pageTree, { runtime: true, identityProviders, totpSetup, formError, isSubmitting })}
    </form>
  )

  if (!layoutIsValid) {
    return (
      <div style={cssVars} onClick={handlePortalActionClick}>
        {responsiveStyle}
        {buttonInteractionStyle}
        {pageContent}
      </div>
    )
  }

  return (
    <div style={cssVars} onClick={handlePortalActionClick}>
      {responsiveStyle}
      {buttonInteractionStyle}
      {treeToReactNode(layoutTree, { runtime: true, pageContent, identityProviders, totpSetup, formError, isSubmitting })}
    </div>
  )
}

// Keep this prefix in sync with the inline pre-paint script in
// `index.html` — both sides need to agree on the key shape, otherwise the
// browser ends up with two competing caches that drift.
const CSSVARS_CACHE_PREFIX = 'fk_portal_theme_cssvars:'

/**
 * Read the previously-persisted CSS vars for `realm`. Used to pre-paint
 * the portal with the last-known theme before the API responds, avoiding a
 * "default → custom" flash on refresh. Returns `null` on parse failure or
 * absence — callers fall back to default-derived vars.
 */
function readCachedCssVars(realm: string): Record<string, string> | null {
  if (typeof window === 'undefined') return null
  try {
    const raw = window.localStorage.getItem(CSSVARS_CACHE_PREFIX + realm)
    if (!raw) return null
    return JSON.parse(raw) as Record<string, string>
  } catch {
    return null
  }
}

function writeCachedCssVars(
  realm: string,
  vars: Record<string, string>,
): void {
  if (typeof window === 'undefined') return
  try {
    window.localStorage.setItem(CSSVARS_CACHE_PREFIX + realm, JSON.stringify(vars))
  } catch {
    // Quota exceeded or storage disabled — silently skip; the cache is a
    // pure perf optimisation, the page still works without it.
  }
}
