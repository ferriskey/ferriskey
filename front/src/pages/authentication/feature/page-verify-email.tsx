import { XCircle, Loader2 } from 'lucide-react'
import { match } from 'ts-pattern'

type VerifyState = 'loading' | 'success' | 'error' | 'expired'

/**
 * React fallback for the `verify_email` portal page. Renders the visual
 * state derived from the verification mutation — loading / success /
 * expired / error.
 *
 * The state is passed in by the parent (`<VerifyEmailRoute>`) so we
 * don't double-fire the verify mutation: this component sits inside
 * `<PortalLayoutWrapper>` and would call `useVerifyEmail` redundantly
 * alongside the route-level call. One mutation per token is critical —
 * the second one would always 4xx with "token already used".
 *
 * Default `state` of `'loading'` covers the case where the fallback is
 * rendered standalone (no parent state) — extremely unlikely in
 * practice but keeps the component robust.
 */
export default function PageVerifyEmail({
  state = 'loading',
}: {
  state?: VerifyState
}) {
  return (
    <div className='flex flex-col items-center justify-center min-h-screen p-4'>
      <div className='max-w-md w-full space-y-6 text-center'>
        {match(state)
          .with('loading', () => (
            <>
              <Loader2 className='w-12 h-12 animate-spin mx-auto text-primary' />
              <p>Verifying your email...</p>
            </>
          ))
          .with('success', () => (
            <>
              <Loader2 className='w-12 h-12 animate-spin mx-auto text-primary' />
              <p>Finishing up...</p>
            </>
          ))
          .with('expired', () => (
            <>
              <XCircle className='w-16 h-16 mx-auto text-amber-500' />
              <h1 className='text-2xl font-bold'>Link expired</h1>
              <p className='text-muted-foreground'>
                This verification link has expired. Please register again or request a new link.
              </p>
            </>
          ))
          .with('error', () => (
            <>
              <XCircle className='w-16 h-16 mx-auto text-red-500' />
              <h1 className='text-2xl font-bold'>Verification failed</h1>
              <p className='text-muted-foreground'>The verification link is invalid.</p>
            </>
          ))
          .exhaustive()}
      </div>
    </div>
  )
}
