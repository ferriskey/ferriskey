import { useGetLoginSettings } from '@/api/realm.api'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { useEffect } from 'react'
import { useLoginForm } from '../hooks/use-login-form'
import { useMagicLinkAuth } from '../hooks/use-magic-link-auth'
import { useOAuthParams } from '../hooks/use-oauth-params'
import { usePasskeyAuth } from '../hooks/use-passkey-auth'
import { useSessionRefresh } from '../hooks/use-session-refresh'
import PageLogin, { LoginErrorPage } from '../ui/page-login'
export type { AuthenticateSchema } from '../hooks/use-login-form'

export default function PageLoginFeature() {
  const { realm_name, isAuthInitiated, loginError, getAuthParamsFromUrl, getOAuthParams } =
    useOAuthParams()

  const { data: loginSettings } = useGetLoginSettings({ realm: realm_name })

  const { form, onSubmit, errorMessage, isSessionError, resetAuthenticate } = useLoginForm({
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

  const isRedirecting = !isAuthInitiated && !loginError

  const { showFloatingActionBar, countdown, cancelAutoRefresh, restartAuthFlow } =
    useSessionRefresh({
      isRedirecting,
      isSessionError,
      getOAuthParams,
      getAuthParamsFromUrl,
      resetAuthenticate,
    })

  useEffect(() => {
    if (isRedirecting) {
      const { query, realm } = getOAuthParams()
      window.location.href = `${window.apiUrl}/realms/${realm}/protocol/openid-connect/auth?${query}`
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
        title='Session expired'
        description={
          countdown !== null
            ? `Refreshing automatically in ${countdown}s...`
            : 'Restart your session to continue.'
        }
        onCancel={countdown !== null ? cancelAutoRefresh : undefined}
        actions={[
          { label: 'Refresh session', variant: 'default', onClick: () => restartAuthFlow() },
        ]}
      />
    </>
  )
}
