import type { Schemas } from '@/api/api.client'

export type ClientProtocol = Schemas.AuthProtocol

interface ClientProtocolOption {
  value: ClientProtocol
  label: string
  description: string
}

export const CLIENT_PROTOCOL_OPTIONS: ClientProtocolOption[] = [
  {
    value: 'openid-connect',
    label: 'OpenID Connect',
    description: 'Token-based sign-in for web apps, SPAs, mobile apps and APIs.',
  },
  {
    value: 'saml',
    label: 'SAML 2.0',
    description: 'Assertion-based sign-in for applications that only speak SAML.',
  },
]

export const DEFAULT_CLIENT_PROTOCOL: ClientProtocol = 'openid-connect'

export function isSamlClient(protocol: string | undefined): boolean {
  return protocol === 'saml'
}
