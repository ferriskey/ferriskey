import { CLIENT_SCOPES_URL } from '@/routes/router'

export const clientScopeUrl = (realmName: string, scopeId: string) =>
  `${CLIENT_SCOPES_URL(realmName)}/${scopeId}`

export const clientScopeMappersUrl = (realmName: string, scopeId: string) =>
  `${clientScopeUrl(realmName, scopeId)}/mappers`

export const protocolMapperUrl = (realmName: string, scopeId: string, mapperId: string) =>
  `${clientScopeMappersUrl(realmName, scopeId)}/${mapperId}`
