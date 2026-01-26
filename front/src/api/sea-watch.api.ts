import { useQuery } from '@tanstack/react-query'
import { BaseQuery } from '.'

export const useGetSecurityEvents = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/seawatch/v1/security-events', {
      path: {
        realm_name: realm!,
      },
    }).queryOptions,
    enabled: !!realm,
  })
}
