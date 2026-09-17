import { XCircle, Loader2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { match } from 'ts-pattern'
import { AUTH_NAMESPACE } from '../constants'

const VERIFY_STATE = {
  LOADING: 'loading',
  SUCCESS: 'success',
  EXPIRED: 'expired',
  ERROR: 'error',
} as const

type VerifyState = (typeof VERIFY_STATE)[keyof typeof VERIFY_STATE]

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
  state = VERIFY_STATE.LOADING,
}: {
  state?: VerifyState
}) {
  const { t } = useTranslation(AUTH_NAMESPACE)

  return (
    <div className='flex flex-col items-center justify-center min-h-screen p-4'>
      <div className='max-w-md w-full space-y-6 text-center'>
        {match(state)
          .with(VERIFY_STATE.LOADING, () => (
            <>
              <Loader2 className='w-12 h-12 animate-spin mx-auto text-primary' />
              <p>{t('verify_email_state.checking')}</p>
            </>
          ))
          .with(VERIFY_STATE.SUCCESS, () => (
            <>
              <Loader2 className='w-12 h-12 animate-spin mx-auto text-primary' />
              <p>{t('verify_email_state.finishing')}</p>
            </>
          ))
          .with(VERIFY_STATE.EXPIRED, () => (
            <>
              <XCircle className='w-16 h-16 mx-auto text-amber-500' />
              <h1 className='text-2xl font-bold'>{t('verify_email_state.expired.title')}</h1>
              <p className='text-muted-foreground'>
                {t('verify_email_state.expired.description')}
              </p>
            </>
          ))
          .with(VERIFY_STATE.ERROR, () => (
            <>
              <XCircle className='w-16 h-16 mx-auto text-red-500' />
              <h1 className='text-2xl font-bold'>{t('verify_email_state.failed.title')}</h1>
              <p className='text-muted-foreground'>
                {t('verify_email_state.failed.description')}
              </p>
            </>
          ))
          .exhaustive()}
      </div>
    </div>
  )
}
