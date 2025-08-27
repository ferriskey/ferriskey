import { useQuery } from '@tanstack/react-query'
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
