import { type Fetcher } from '@/api/api.client.ts'
import { authStore } from '@/store/auth.store.ts'

export interface BaseQuery {
  realm?: string
}

export const fetcher: Fetcher['fetch'] = async (input) => {
  const headers = new Headers()

  const accessToken = authStore.getState().accessToken

  // Handle query parameters via URLSearchParams
  if (input.urlSearchParams) {
    input.url.search = input.urlSearchParams.toString()
  }

  // Handle request body for mutation methods
  let body: BodyInit | undefined
  if (['post', 'put', 'patch', 'delete'].includes(input.method.toLowerCase()) && input.parameters?.body !== undefined) {
    const bodyData = input.parameters.body
    if (
      bodyData instanceof URLSearchParams ||
      bodyData instanceof FormData ||
      typeof bodyData === 'string' ||
      bodyData instanceof Blob ||
      bodyData instanceof ArrayBuffer
    ) {
      body = bodyData as BodyInit
    } else {
      body = JSON.stringify(bodyData)
      headers.set('Content-Type', 'application/json')
    }
  }

  if (accessToken) {
    headers.set('Authorization', `Bearer ${accessToken}`)
  }

  // Add custom headers
  if (input.parameters?.header) {
    Object.entries(input.parameters.header).forEach(([key, value]) => {
      if (value != null) {
        headers.set(key, String(value))
      }
    })
  }

  const response = await fetch(input.url, {
    method: input.method.toUpperCase(),
    ...(body && { body }),
    headers,
    credentials: 'include',
    ...input.overrides,
  })

  if (!response.ok) {
    let errorBody: Record<string, unknown> | undefined
    try {
      errorBody = await response.json()
    } catch {
      // If parsing fails, errorBody stays undefined
    }

    const error: Error & { status?: number; data?: Record<string, unknown> } = new Error(
      (errorBody?.message as string) ?? `HTTP ${response.status}: ${response.statusText}`
    )
    error.status = response.status
    error.data = errorBody
    throw error
  }

  return response
}
