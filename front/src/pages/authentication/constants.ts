import type { Schemas } from '@/api/api.client'
import { preloadNamespaces } from '@/lib/i18n'

export const AUTH_NAMESPACE = 'auth'

void preloadNamespaces(AUTH_NAMESPACE).catch(() => undefined)

export const BRAND_NAME = 'FerrisKey'

export const PORTAL_PAGE_TYPE = {
  LOGIN: 'login',
  REGISTER: 'register',
  TOTP: 'totp',
  TOTP_SETUP: 'totp_setup',
  FORGOT_PASSWORD: 'forgot_password',
  RESET_PASSWORD: 'reset_password',
  MAGIC_LINK_VERIFY: 'magic_link_verify',
  MAGIC_LINK_REQUEST: 'magic_link_request',
  VERIFY_EMAIL: 'verify_email',
  EMAIL_VERIFIED: 'email_verified',
  DEVICE_VERIFY: 'device_verify',
} as const satisfies Record<string, Schemas.PortalPageType>

export const DEVICE_ACTION = {
  APPROVE: 'approve',
  DENY: 'deny',
} as const

export type DeviceAction = (typeof DEVICE_ACTION)[keyof typeof DEVICE_ACTION]
