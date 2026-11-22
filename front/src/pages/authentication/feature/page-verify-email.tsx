import { XCircle, Loader2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { match } from 'ts-pattern'
import AuthShell, { AuthShellCenter } from '../components/auth-shell'
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

  return match(state)
    .with(VERIFY_STATE.LOADING, () => (
      <AuthShellCenter>
        <div className='relative z-10 w-full max-w-md space-y-6 text-center'>
          <Loader2 className='w-12 h-12 animate-spin mx-auto text-primary' />
          <p>{t('verify_email_state.checking')}</p>
        </div>
      </AuthShellCenter>
    ))
    .with(VERIFY_STATE.SUCCESS, () => (
      <AuthShellCenter>
        <div className='relative z-10 w-full max-w-md space-y-6 text-center'>
          <Loader2 className='w-12 h-12 animate-spin mx-auto text-primary' />
          <p>{t('verify_email_state.finishing')}</p>
        </div>
      </AuthShellCenter>
    ))
    .with(VERIFY_STATE.EXPIRED, () => (
      <AuthShell
        title={t('verify_email_state.expired.title')}
        description={t('verify_email_state.expired.description')}
      >
        <XCircle className='w-16 h-16 mx-auto text-amber-500' />
      </AuthShell>
    ))
    .with(VERIFY_STATE.ERROR, () => (
      <AuthShell
        title={t('verify_email_state.failed.title')}
        description={t('verify_email_state.failed.description')}
      >
        <XCircle className='w-16 h-16 mx-auto text-red-500' />
      </AuthShell>
    ))
    .exhaustive()
}
