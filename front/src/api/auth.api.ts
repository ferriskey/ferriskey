import { createApi } from '@reduxjs/toolkit/query/react'
import { baseQuery } from './index'

export const authApi = createApi({
  reducerPath: 'auth',
  baseQuery,
  endpoints: (builder) => ({
    authenticate: builder.mutation({
      query: (payload) => ({
        url: `/realms/${payload.realm}/login-actions/authenticate`,
        method: 'POST',
        body: payload.data,
      }),
    }),
  }),
})
