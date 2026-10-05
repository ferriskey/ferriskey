export type ApplicationType = 'native' | 'spa' | 'web' | 'm2m' | 'device'

export interface ApplicationShape {
  service_account_enabled: boolean
  oauth_device_code_grant_enabled: boolean
  client_type: string
  public_client: boolean
  redirect_uris?: readonly unknown[] | null
}

export function inferApplicationType(client: ApplicationShape): ApplicationType {
  if (client.service_account_enabled) return 'm2m'
  if (client.oauth_device_code_grant_enabled && (client.redirect_uris?.length ?? 0) === 0) {
    return 'device'
  }
  if (client.client_type === 'public') return client.public_client ? 'spa' : 'native'
  return 'web'
}
