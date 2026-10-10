import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from './api.client'

export type ResourceOwner = Schemas.ResourceOwner

const resourceOwnersKey = (realm: string) =>
  window.tanstackApi.get('/realms/{realm_name}/resource-owners', {
    path: { realm_name: realm },
  }).queryKey

export const useGetResourceOwners = ({ realm }: { realm?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/resource-owners', {
      path: { realm_name: realm ?? '' },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useUpdateResourceOwners = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ realm, owners }: { realm: string; owners: ResourceOwner[] }) =>
      window.api.put('/realms/{realm_name}/resource-owners', {
        path: { realm_name: realm },
        body: { owners },
      }),
    onSuccess: async (_response, variables) => {
      await queryClient.invalidateQueries({ queryKey: resourceOwnersKey(variables.realm) })
    },
  })
}
