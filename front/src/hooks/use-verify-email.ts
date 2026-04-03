import { useEffect, useState, useRef } from 'react'
import { useSearchParams, useParams } from 'react-router'

type VerifyState = 'loading' | 'success' | 'error' | 'expired'

export function useVerifyEmail() {
  const [searchParams] = useSearchParams()
  const { realm_name } = useParams()
  const hasFetched = useRef(false)

  const token = searchParams.get('token')
  const initialState: VerifyState = token && realm_name ? 'loading' : 'error'
  const [state, setState] = useState<VerifyState>(initialState)

  useEffect(() => {
    if (!token || !realm_name || hasFetched.current) return
    hasFetched.current = true

    const apiUrl = window.apiUrl || ''
    fetch(
      `${apiUrl}/realms/${encodeURIComponent(realm_name)}/login-actions/verify-email?token=${encodeURIComponent(token)}`
    )
      .then((res) => {
        if (res.ok) return setState('success')
        if (res.status === 400 || res.status === 410) return setState('expired')
        setState('error')
      })
      .catch(() => setState('error'))
  }, [token, realm_name])

  return { state, realm_name }
}
