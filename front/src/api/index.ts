import { BaseQueryFn, FetchArgs, fetchBaseQuery, FetchBaseQueryError } from '@reduxjs/toolkit/query'

export const backendUrl = import.meta.env.VITE_API_URL

export const baseQuery: BaseQueryFn<string | FetchArgs, unknown, FetchBaseQueryError> = async (
  args,
  api,
  extraOptions
) => {
  const baseQuery = fetchBaseQuery({
    baseUrl: backendUrl,
    prepareHeaders: (headers) => {
      const token = sessionStorage.getItem('token')
      headers.set('Authorization', `Bearer ${token}`)
    },
  })

  const result = await baseQuery(args, api, extraOptions)
  return result
}
