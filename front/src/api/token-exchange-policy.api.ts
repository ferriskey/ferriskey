import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { apiErrorMessage, apiErrorPayload, type ApiRequestError } from '@/lib/api-error'
import { translate } from '@/lib/i18n'

const POLICIES_PATH = '/realms/{realm_name}/clients/{client_id}/token-exchange-policies'

const CONFLICT = 409

const invalidatePolicies = (
  queryClient: ReturnType<typeof useQueryClient>,
  realmName: string,
  clientId: string
) =>
  queryClient.invalidateQueries({
    queryKey: window.tanstackApi.get(POLICIES_PATH, {
      path: { realm_name: realmName, client_id: clientId },
    }).queryKey,
  })

export const useGetTokenExchangePolicies = ({
  realm,
  clientId,
}: {
  realm?: string
  clientId?: string
}) => {
  return useQuery({
    ...window.tanstackApi.get(POLICIES_PATH, {
      path: { realm_name: realm!, client_id: clientId! },
    }).queryOptions,
    enabled: !!realm && !!clientId,
  })
}

export const useCreateTokenExchangePolicy = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', POLICIES_PATH).mutationOptions,
    onSuccess: async (_, variables) => {
      await invalidatePolicies(queryClient, variables.path.realm_name, variables.path.client_id)
      toast.success(translate('client:token_exchange.toast.created'))
    },
    onError: (error) => {
      const status = apiErrorPayload(error)?.status ?? (error as ApiRequestError).status

      if (status === CONFLICT) {
        toast.error(translate('client:token_exchange.toast.conflict'))
        return
      }

      toast.error(translate('client:token_exchange.toast.create_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useDeleteTokenExchangePolicy = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('delete', `${POLICIES_PATH}/{policy_id}`).mutationOptions,
    onSuccess: async (_, variables) => {
      await invalidatePolicies(queryClient, variables.path.realm_name, variables.path.client_id)
      toast.success(translate('client:token_exchange.toast.deleted'))
    },
    onError: (error) => {
      toast.error(translate('client:token_exchange.toast.delete_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}
