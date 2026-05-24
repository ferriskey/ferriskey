import { useEffect, useRef, useState } from 'react'
import { useSearchParams, useParams } from 'react-router'
import { useVerifyEmailMutation } from '@/api/auth.api'

type VerifyState = 'loading' | 'success' | 'error' | 'expired'

export function useVerifyEmail() {
  const [searchParams] = useSearchParams()
  const { realm_name } = useParams()
  const submittedKey = useRef<string | null>(null)
  const { mutateAsync } = useVerifyEmailMutation()

  const token = searchParams.get('token')

  // Derive initial state from URL params
  const hasRequiredParams = Boolean(token && realm_name)
  const [state, setState] = useState<VerifyState>(hasRequiredParams ? 'loading' : 'error')

  useEffect(() => {
    if (!token || !realm_name) return

    const requestKey = `${realm_name}:${token}`
    if (submittedKey.current === requestKey) return

    submittedKey.current = requestKey

    mutateAsync({
      path: { realm_name },
      body: { token },
    })
      .then(() => {
        setState('success')
      })
      .catch((err: Error & { status?: number }) => {
        if (err.status === 400 || err.status === 410) {
          setState('expired')
        } else {
          setState('error')
        }
      })
  }, [mutateAsync, realm_name, token])

  return { state, realm_name }
}
