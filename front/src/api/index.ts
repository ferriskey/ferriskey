import { Fetcher } from '@/api/api.client.ts'
import { authStore } from '@/store/auth.store.ts'

export interface BaseQuery {
  realm?: string
}

export const fetcher: Fetcher = async (method, apiUrl, params) => {
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
  let body: BodyInit | undefined
  if (['post', 'put', 'patch', 'delete'].includes(method.toLowerCase()) && params?.body !== undefined) {
    if (
      params.body instanceof URLSearchParams ||
      params.body instanceof FormData ||
      typeof params.body === 'string' ||
      params.body instanceof Blob ||
      params.body instanceof ArrayBuffer
    ) {
      body = params.body as BodyInit
    } else {
      body = JSON.stringify(params.body)
      headers.set('Content-Type', 'application/json')
    }
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
    credentials: 'include',
  })

  if (!response.ok) {
    // Parse the error response to get the message from the backend
    let errorMessage = `HTTP ${response.status}: ${response.statusText}`
    try {
      const errorBody = await response.json()
      if (errorBody.message) {
        errorMessage = errorBody.message
      }
    } catch {
      // If parsing fails, use the default message
    }

    throw new Error(errorMessage)
  }

  return response
}

function replacePathParams(url: string, params: Record<string, string>): string {
  return url
    .replace(/{(\w+)}/g, (_, key: string) => params[key] || `{${key}}`)
    .replace(/:([a-zA-Z0-9_]+)/g, (_, key: string) => params[key] || `:${key}`)
}
