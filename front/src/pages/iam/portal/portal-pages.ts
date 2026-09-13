import type { Schemas } from '@/api/api.client'

export type PortalPageType = Schemas.PortalPageType

export interface PortalPageDescriptor {
  type: PortalPageType
  label: string
  description: string
}

export const PORTAL_PAGES: PortalPageDescriptor[] = [
  {
    type: 'login',
    label: 'Login',
    description: 'Credentials, identity providers and the entry point of every flow.',
  },
  {
    type: 'register',
    label: 'Register',
    description: 'Self-service account creation, when the realm allows it.',
  },
  {
    type: 'totp',
    label: 'OTP challenge',
    description: 'Second factor asked after a successful password check.',
  },
  {
    type: 'forgot_password',
    label: 'Forgot password',
    description: 'Asks for an address and sends the reset mail.',
  },
  {
    type: 'reset_password',
    label: 'Reset password',
    description: 'Reached from the mail link; sets the new password.',
  },
  {
    type: 'magic_link_request',
    label: 'Magic link request',
    description: 'Asks for an address and sends a passwordless sign-in link.',
  },
  {
    type: 'magic_link_verify',
    label: 'Magic link verify',
    description: 'Landing page of the link, where the session is opened.',
  },
  {
    type: 'verify_email',
    label: 'Verify email',
    description: 'Asks the user to confirm the address they registered with.',
  },
  {
    type: 'email_verified',
    label: 'Email verified',
    description: 'Confirmation screen once the address is proven.',
  },
  {
    type: 'totp_setup',
    label: 'TOTP setup',
    description: 'First-time enrolment: QR code, secret and confirmation code.',
  },
  {
    type: 'device_verify',
    label: 'Device verification',
    description: 'RFC 8628 consent screen: the user code, approve and deny.',
  },
  {
    type: 'device_verified',
    label: 'Device verified',
    description: 'Confirmation screen; the device signs in on its own.',
  },
]

export function labelForPortalPage(pageType: PortalPageType): string {
  return PORTAL_PAGES.find((p) => p.type === pageType)?.label ?? pageType
}

export function humanizeBlockType(type: string): string {
  return type.replace(/_/g, ' ')
}
