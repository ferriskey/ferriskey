import { useEffect, useMemo, useRef } from 'react'
import { useSearchParams, useParams } from 'react-router'
import { TypedStatusError } from '@/api/api.client'
import { useVerifyEmailMutation } from '@/api/auth.api'

type VerifyState = 'loading' | 'success' | 'error' | 'expired'

export function useVerifyEmail() {
  const [searchParams] = useSearchParams()
  const { realm_name } = useParams()
  const submittedKey = useRef<string | null>(null)
  const { mutate, isSuccess, isError, error, reset } = useVerifyEmailMutation()

  const token = searchParams.get('token')
  const requestKey = token && realm_name ? `${realm_name}:${token}` : null

  useEffect(() => {
    if (!requestKey || !token || !realm_name) {
      submittedKey.current = null
      reset()
      return
    }

    if (submittedKey.current === requestKey) return

    submittedKey.current = requestKey
    mutate({
      path: { realm_name },
      body: { token },
    })
  }, [mutate, realm_name, requestKey, reset, token])

  const state = useMemo<VerifyState>(() => {
    if (!token || !realm_name) return 'error'
    if (isSuccess) return 'success'
    if (isError) {
      if (error instanceof TypedStatusError && (error.status === 400 || error.status === 410)) {
        return 'expired'
      }

      return 'error'
    }

    return 'loading'
  }, [error, isError, isSuccess, realm_name, token])

  return { state, realm_name }
}
