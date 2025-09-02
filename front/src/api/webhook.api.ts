import { useMutation, useQuery } from '@tanstack/react-query'
import { BaseQuery } from '.'

export const useGetWebhooks = ({ realm = 'master' }: BaseQuery) => {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/webhooks', {
      path: {
        realm_name: realm,
      },
    }).queryOptions
  )
}

export const useCreateWebhook = () => {
  return useMutation(
    window.tanstackApi.mutation('post', '/realms/{realm_name}/webhooks', async (res) => res.json())
      .mutationOptions
  )
}
