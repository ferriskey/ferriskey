import {
  BaseQueryFn,
  FetchArgs,
  fetchBaseQuery,
  FetchBaseQueryError,
} from '@reduxjs/toolkit/query/react'
import axios from 'axios'

export const backendUrl = import.meta.env.VITE_API_URL ?? 'http://localhost:3333'

export const baseQuery: BaseQueryFn<string | FetchArgs, unknown, FetchBaseQueryError> = async (
  args,
  api,
  extraOptions
) => {
  const baseQuery = fetchBaseQuery({
    baseUrl: backendUrl,
    credentials: 'include',
    prepareHeaders: (headers) => {
      const token = sessionStorage.getItem('token')
      headers.set('Authorization', `Bearer ${token}`)
    },
  })

  const result = await baseQuery(args, api, extraOptions)

  console.log(result)

  return result
}

const apiUrl = 'http://localhost:3333'

export interface BaseQuery {
  realm: string
}

const defaultHeaders = {
  'Content-Type': 'application/json',
}

export const apiClient = axios.create({
  baseURL: apiUrl,
  headers: defaultHeaders,
  withCredentials: true,
})
