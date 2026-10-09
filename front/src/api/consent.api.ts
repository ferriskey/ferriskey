import { useMutation, useQuery } from '@tanstack/react-query'
import type { Schemas } from './api.client'
import type { ConsentScopeView } from '@/pages/authentication/utils/consent-decision'

export function toConsentScopeList(scopes: Schemas.ScopeView[]): ConsentScopeView[] {
  return scopes.map((scope) => ({ name: scope.name, description: scope.description ?? null }))
}

export function useGetConsentRequest({
  realm,
  consentToken,
  enabled,
}: {
  realm: string
  consentToken: string
  enabled: boolean
}) {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/auth/consent', {
      path: { realm_name: realm },
      query: { consent_token: consentToken },
    }).queryOptions,
    enabled: enabled && consentToken.length > 0,
    retry: false,
    staleTime: 0,
  })
}

export function useSubmitConsent() {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/auth/consent').mutationOptions,
  })
}
