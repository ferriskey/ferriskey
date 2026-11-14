import { useMutation, useQuery } from '@tanstack/react-query'

const accountsRequest = (realm: string, clientId: string, maxAge?: number) =>
  window.tanstackApi.get('/realms/{realm_name}/login-actions/accounts', {
    path: { realm_name: realm },
    query: { client_id: clientId, max_age: maxAge },
  })

export const accountsQueryKey = (realm: string, clientId: string, maxAge?: number) =>
  accountsRequest(realm, clientId, maxAge).queryKey

export const useGetAccounts = ({
  realm,
  clientId,
  maxAge,
}: {
  realm: string
  clientId: string
  maxAge?: number
}) => {
  return useQuery({
    ...accountsRequest(realm, clientId, maxAge).queryOptions,
    retry: false,
    staleTime: 0,
    gcTime: 0,
    refetchOnWindowFocus: false,
  })
}

export const useSelectAccount = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/login-actions/accounts/select')
      .mutationOptions,
  })
}

export const useRemoveAccount = () => {
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/login-actions/accounts/{session_id}')
      .mutationOptions,
  })
}
