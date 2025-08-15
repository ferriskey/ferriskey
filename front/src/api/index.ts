import axios from 'axios'
import { createApiClient, Fetcher } from '@/api/api.client.ts'
import { TanstackQueryApiClient } from '@/api/api.tanstack.ts'
import { authStore } from '@/store/auth.store.ts'

export const apiUrl = import.meta.env.VITE_API_URL ?? 'http://localhost:3333'

export interface BaseQuery {
  realm?: string
}

const defaultHeaders = {
  'Content-Type': 'application/json',
}

export const apiClient = axios.create({
  baseURL: apiUrl,
  headers: defaultHeaders,
  withCredentials: true,
})

const fetcher: Fetcher = async (method, apiUrl, params) => {
  const headers = new Headers()

  const accessToken = authStore.getState().accessToken

  // Replace path parameters (supports both {param} and :param formats)
  const actualUrl = replacePathParams(apiUrl, (params?.path ?? {}) as Record<string, string>)
  const url = new URL(actualUrl)

  // Handle query parameters
  if (params?.query) {
    const searchParams = new URLSearchParams()
    Object.entries(params.query).forEach(([key, value]) => {
      if (value != null) {
        // Skip null/undefined values
        if (Array.isArray(value)) {
          value.forEach((val) => val != null && searchParams.append(key, String(val)))
        } else {
          searchParams.append(key, String(value))
        }
      }
    })
    url.search = searchParams.toString()
  }

  // Handle request body for mutation methods
  const body = ['post', 'put', 'patch', 'delete'].includes(method.toLowerCase())
    ? JSON.stringify(params?.body)
    : undefined

  if (body) {
    headers.set('Content-Type', 'application/json')
  }

  if (accessToken) {
    headers.set('Authorization', `Bearer ${accessToken}`)
  }

  // Add custom headers
  if (params?.header) {
    Object.entries(params.header).forEach(([key, value]) => {
      if (value != null) {
        headers.set(key, String(value))
      }
    })
  }

  const response = await fetch(url, {
    method: method.toUpperCase(),
    ...(body && { body }),
    headers,
  })

  if (!response.ok) {
    // You can customize error handling here
    const error = new Error(`HTTP ${response.status}: ${response.statusText}`)
    throw error
  }

  return response
}

function replacePathParams(url: string, params: Record<string, string>): string {
  return url
    .replace(/{(\w+)}/g, (_, key: string) => params[key] || `{${key}}`)
    .replace(/:([a-zA-Z0-9_]+)/g, (_, key: string) => params[key] || `:${key}`)
}

const api = createApiClient(fetcher, apiUrl)

export const tanstackApi = new TanstackQueryApiClient(api)
