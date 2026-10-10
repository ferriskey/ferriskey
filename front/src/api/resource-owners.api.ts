import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { z } from 'zod'

// TODO: switch to the generated hooks once api.client.ts is regenerated with /resource-owners.
const RESOURCE_OWNERS_PATH = '/realms/{realm_name}/resource-owners'

const resourceOwnerSchema = z.object({
  uri: z.string(),
  client_id: z.string(),
})

const resourceOwnersResponseSchema = z.object({
  data: z.array(resourceOwnerSchema),
})

export type ResourceOwner = z.infer<typeof resourceOwnerSchema>
export type ResourceOwnersResponse = z.infer<typeof resourceOwnersResponseSchema>

interface ResourceOwnersRequestClient {
  request: (
    method: 'get' | 'put',
    path: string,
    params: { path: { realm_name: string }; body?: { owners: ResourceOwner[] } }
  ) => Promise<unknown>
}

const resourceOwnersKey = (realm: string) => ['resource-owners', realm] as const

const requestClient = () => window.api as unknown as ResourceOwnersRequestClient

export const useGetResourceOwners = ({ realm }: { realm?: string }) => {
  return useQuery({
    queryKey: resourceOwnersKey(realm ?? ''),
    queryFn: async () =>
      resourceOwnersResponseSchema.parse(
        await requestClient().request('get', RESOURCE_OWNERS_PATH, {
          path: { realm_name: realm! },
        })
      ),
    enabled: !!realm,
  })
}

export const useUpdateResourceOwners = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ realm, owners }: { realm: string; owners: ResourceOwner[] }) =>
      resourceOwnersResponseSchema.parse(
        await requestClient().request('put', RESOURCE_OWNERS_PATH, {
          path: { realm_name: realm },
          body: { owners },
        })
      ),
    onSuccess: async (_response, variables) => {
      await queryClient.invalidateQueries({ queryKey: resourceOwnersKey(variables.realm) })
    },
  })
}
