import { useGetLoginSettings } from '@/api/realm.api'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useLoginForm } from '../hooks/use-login-form'
import { useMagicLinkAuth } from '../hooks/use-magic-link-auth'
import { useOAuthParams } from '../hooks/use-oauth-params'
import { usePasskeyAuth } from '../hooks/use-passkey-auth'
import { useSessionRefresh } from '../hooks/use-session-refresh'
import PageLogin, { LoginErrorPage } from '../ui/page-login'
import { useAuth } from '@/hooks/use-auth'
import { useNavigate } from 'react-router'
import { AUTH_NAMESPACE } from '../constants'
import { resolveLoginSession } from './login-session'
export type { AuthenticateSchema } from '../hooks/use-login-form'

export default function PageLoginFeature() {
  const navigate = useNavigate()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const { isAuthenticated, clearAuthState } = useAuth()
  const { realm_name, sessionExpired, isAuthInitiated, loginError, getAuthParamsFromUrl, getOAuthParams } =
    useOAuthParams()

  const sessionAction = resolveLoginSession({ isAuthenticated, isAuthInitiated, sessionExpired })

  useEffect(() => {
    if (sessionAction === 'discard-stale-session') {
      clearAuthState(false)
      return
    }

    if (sessionAction === 'enter-console') {
      navigate(`/realms/${realm_name}/overview`, { replace: true })
    }
  }, [sessionAction, clearAuthState, navigate, realm_name])

  const { data: loginSettings } = useGetLoginSettings({ realm: realm_name })

  const { form, onSubmit, errorMessage, isSessionError, isMaintenanceError } =
    useLoginForm({
      realm_name,
      loginError,
      getAuthParamsFromUrl,
    })

  const { onPasskeyLogin, isPasskeyLoading } = usePasskeyAuth({
    realm_name,
    enabled: Boolean(loginSettings?.passkey_enabled),
    isAuthInitiated,
  })

  const {
    magicLinkForm,
    magicLinkStep,
    isMagicLinkLoading,
    onMagicLinkLogin,
    onMagicLinkBack,
    onMagicLinkSubmit,
  } = useMagicLinkAuth({ realm_name })

  const isRedirecting = !isAuthInitiated && !loginError && !sessionExpired

  const { showFloatingActionBar, countdown, cancelAutoRefresh, restartAuthFlow } =
    useSessionRefresh({
      isRedirecting,
      isSessionError,
      getOAuthParams,
    })

  useEffect(() => {
    if (isRedirecting) {
      void getOAuthParams().then(({ query, realm }) => {
        window.location.href = `${window.apiUrl}/realms/${realm}/protocol/openid-connect/auth?${query}`
      })
    }
  }, [isRedirecting, getOAuthParams])

  if (isRedirecting) {
    return <PageLogin form={form} onSubmit={onSubmit} isLoading loginSettings={loginSettings} />
  }

  // Fatal configuration error (e.g. "Invalid redirect URI", "Client not found").
  // The backend redirected here because it can't trust the redirect_uri.
  // Show a clean error card — do NOT render the login form or retry the OAuth flow.
  if (loginError && !isAuthInitiated) {
    return <LoginErrorPage errorMessage={loginError} />
  }

  if (!loginSettings) return null

  return (
    <>
      <PageLogin
        form={form}
        onSubmit={onSubmit}
        isError={undefined}
        loginSettings={loginSettings}
        errorMessage={errorMessage}
        isMaintenanceError={isMaintenanceError}
        onPasskeyLogin={loginSettings?.passkey_enabled ? onPasskeyLogin : undefined}
        isPasskeyLoading={isPasskeyLoading}
        onMagicLinkLogin={loginSettings?.magic_link_enabled ? onMagicLinkLogin : undefined}
        isMagicLinkLoading={isMagicLinkLoading}
        magicLinkStep={loginSettings?.magic_link_enabled ? magicLinkStep : undefined}
        magicLinkForm={magicLinkForm}
        onMagicLinkSubmit={onMagicLinkSubmit}
        onMagicLinkBack={onMagicLinkBack}
      />
      <FloatingActionBar
        show={showFloatingActionBar}
        title={t('session.expired')}
        description={
          countdown !== null
            ? t('session.refreshing', { seconds: countdown })
            : t('session.restart')
        }
        onCancel={countdown !== null ? cancelAutoRefresh : undefined}
        actions={[
          { label: t('session.refresh'), variant: 'default', onClick: () => restartAuthFlow() },
        ]}
      />
    </>
  )
}
