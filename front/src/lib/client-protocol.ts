import type { Schemas } from '@/api/api.client'
import { translate } from '@/lib/i18n'

export type ClientProtocol = Schemas.AuthProtocol

interface ClientProtocolOption {
  value: ClientProtocol
  label: string
  description: string
}

const PROTOCOL_KEYS: Record<ClientProtocol, string> = {
  'openid-connect': 'oidc',
  saml: 'saml',
}

export const CLIENT_PROTOCOL_OPTIONS: ClientProtocolOption[] = (
  Object.keys(PROTOCOL_KEYS) as ClientProtocol[]
).map((value) => ({
  value,
  get label() {
    return translate(`common:client_protocol.${PROTOCOL_KEYS[value]}.label`)
  },
  get description() {
    return translate(`common:client_protocol.${PROTOCOL_KEYS[value]}.description`)
  },
}))

export const DEFAULT_CLIENT_PROTOCOL: ClientProtocol = 'openid-connect'

export function isSamlClient(protocol: string | undefined): boolean {
  return protocol === 'saml'
}
