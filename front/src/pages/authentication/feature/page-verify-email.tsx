import { useEffect, useState, useRef } from 'react'
import { useSearchParams, useParams, Link } from 'react-router'
import { CheckCircle, XCircle, Loader2 } from 'lucide-react'

type VerifyState = 'loading' | 'success' | 'error' | 'expired'

export default function PageVerifyEmail() {
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

  return (
    <div className='flex flex-col items-center justify-center min-h-screen p-4'>
      <div className='max-w-md w-full space-y-6 text-center'>
        {state === 'loading' && (
          <>
            <Loader2 className='w-12 h-12 animate-spin mx-auto text-primary' />
            <p>Verifying your email...</p>
          </>
        )}

        {state === 'success' && (
          <>
            <CheckCircle className='w-16 h-16 mx-auto text-green-600' />
            <h1 className='text-2xl font-bold'>Email verified!</h1>
            <p className='text-muted-foreground'>
              Your email has been verified. You can now sign in.
            </p>
            <Link
              to={`/realms/${realm_name}/authentication/login`}
              className='inline-block px-6 py-2 bg-primary text-primary-foreground rounded-md'
            >
              Go to login
            </Link>
          </>
        )}

        {state === 'expired' && (
          <>
            <XCircle className='w-16 h-16 mx-auto text-amber-500' />
            <h1 className='text-2xl font-bold'>Link expired</h1>
            <p className='text-muted-foreground'>
              This verification link has expired. Please register again or request a new link.
            </p>
          </>
        )}

        {state === 'error' && (
          <>
            <XCircle className='w-16 h-16 mx-auto text-red-500' />
            <h1 className='text-2xl font-bold'>Verification failed</h1>
            <p className='text-muted-foreground'>
              The verification link is invalid.
            </p>
          </>
        )}
      </div>
    </div>
  )
}
