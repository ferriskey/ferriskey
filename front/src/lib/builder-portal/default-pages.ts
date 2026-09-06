import type { BuilderNode } from '@/lib/builder-core'
import type { Schemas } from '@/api/api.client'
import { PORTAL_PRESETS } from './presets'

type PageType = Schemas.PortalPageType

/**
 * The preset(s) a freshly created theme starts each page with.
 *
 * A theme created with empty pages cannot be activated: the server refuses
 * activation until every page carries the blocks its flow needs (an email and
 * password field on Login, a code field on TOTP, and so on), so an admin had to
 * compose twelve pages by hand before their theme could go live. Seeding each
 * page from the matching preset means a new theme is activatable as it stands,
 * and the admin edits from something rather than from nothing.
 *
 * Login takes two presets: the sign-in card carries the credentials form, and
 * `or-continue-with` appends the identity providers block the page requires.
 */
const PAGE_PRESET_IDS: Record<PageType, string[]> = {
  login: ['sign-in-card', 'or-continue-with'],
  register: ['register-card'],
  totp: ['totp-card'],
  forgot_password: ['forgot-password-card'],
  reset_password: ['reset-password-card'],
  magic_link_request: ['magic-link-request-card'],
  magic_link_verify: ['magic-link-verify-card'],
  verify_email: ['verify-email-card'],
  email_verified: ['email-verified-card'],
  totp_setup: ['totp-setup-card'],
  device_verify: ['device-verify-card'],
  device_verified: ['device-verified-card'],
}

export const DEFAULT_PAGE_TYPES = Object.keys(PAGE_PRESET_IDS) as PageType[]

/**
 * Builds the starting tree for one page. Presets are factories because every
 * node needs a fresh id, so this must be called per theme, never cached.
 */
export function defaultPageTree(pageType: PageType): BuilderNode[] {
  return PAGE_PRESET_IDS[pageType].flatMap((presetId) => {
    const preset = PORTAL_PRESETS.find((candidate) => candidate.id === presetId)
    // A preset renamed out from under this table would silently produce an
    // empty page, which is exactly the un-activatable state this exists to
    // prevent — so say so instead of shipping the hole.
    if (!preset) {
      throw new Error(`unknown portal preset "${presetId}" for page "${pageType}"`)
    }
    return preset.factory()
  })
}
