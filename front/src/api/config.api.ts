import { useQuery } from '@tanstack/react-query'
import { GetConfigResponse } from './api.interface'

export const useGetConfig = () => {
  return useQuery({
    queryKey: ['config'],
    queryFn: async (): Promise<GetConfigResponse> => {
      const response = await window.axios.get<GetConfigResponse>('/config')

      return response.data
    },
  })
}
