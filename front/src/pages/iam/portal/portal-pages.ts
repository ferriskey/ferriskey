import type { Schemas } from '@/api/api.client'
import { preloadNamespaces, translate } from '@/lib/i18n'

export const PORTAL_NAMESPACE = 'portal'

void preloadNamespaces(PORTAL_NAMESPACE).catch(() => undefined)

export type PortalPageType = Schemas.PortalPageType

export interface PortalPageDescriptor {
  type: PortalPageType
  labelKey: string
  descriptionKey: string
}

const PORTAL_PAGE_TYPES: PortalPageType[] = [
  'login',
  'register',
  'totp',
  'forgot_password',
  'reset_password',
  'magic_link_request',
  'magic_link_verify',
  'verify_email',
  'email_verified',
  'totp_setup',
  'device_verify',
  'device_verified',
]

export const PORTAL_PAGES: PortalPageDescriptor[] = PORTAL_PAGE_TYPES.map((type) => ({
  type,
  labelKey: `page_type.${type}.label`,
  descriptionKey: `page_type.${type}.description`,
}))

export function labelForPortalPage(pageType: PortalPageType): string {
  const page = PORTAL_PAGES.find((p) => p.type === pageType)
  return page ? translate(`${PORTAL_NAMESPACE}:${page.labelKey}`) : pageType
}

export function humanizeBlockType(type: string): string {
  return type.replace(/_/g, ' ')
}
