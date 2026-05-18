import { useCallback, useEffect } from 'react'
import { useNavigate } from 'react-router'
import { AuthenticationStatus } from '@/api/api.interface'
import { useAuthenticateMutation } from '@/api/auth.api'
import type { Schemas } from '@/api/api.client'
import { useOAuthParams } from './use-oauth-params'

/**
 * Returns an `onSubmit(FormData)` handler tailored to the portal page type.
 * The wrapper passes this to the renderer when the realm admin has built a
 * custom page tree; the renderer wraps the tree in a <form> and the submit
 * block triggers this handler with the inputs' name-keyed values.
 *
 * Only `login` is wired today — other page types return `undefined` and the
 * <form> submits to nothing (no-op) until follow-up PRs hook them up.
 */
export function usePortalPageSubmit(pageType: Schemas.PortalPageType): {
  onSubmit?: (data: FormData) => void
} {
  const { realm_name, getAuthParamsFromUrl } = useOAuthParams()
  const navigate = useNavigate()
  const {
    mutate: authenticate,
    data: authenticateData,
  } = useAuthenticateMutation()

  // Mirror the post-success navigation from useLoginForm so the custom-tree
  // login flow ends up at the same callback / required-action / OTP screens.
  useEffect(() => {
    if (pageType !== 'login' || !authenticateData) return
    if (authenticateData.url) {
      window.location.href = authenticateData.url
      return
    }
    if (
      authenticateData.status === AuthenticationStatus.RequiresActions &&
      authenticateData.required_actions &&
      authenticateData.required_actions.length > 0 &&
      authenticateData.token
    ) {
      const first = authenticateData.required_actions[0]
      navigate(
        `/realms/${realm_name}/authentication/required-action?execution=${first.toUpperCase()}&client_data=${authenticateData.token}`,
      )
      return
    }
    if (authenticateData.status === AuthenticationStatus.RequiresOtpChallenge) {
      navigate(
        `/realms/${realm_name}/authentication/otp?token=${authenticateData.token}`,
      )
    }
  }, [pageType, authenticateData, navigate, realm_name])

  const loginSubmit = useCallback(
    (data: FormData) => {
      // Map the conventional block names back to the API's payload. An
      // `email_input` defaults to name="email" but the auth endpoint expects
      // `username`, so accept either field name.
      const username = String(data.get('email') ?? data.get('username') ?? '').trim()
      const password = String(data.get('password') ?? '')
      if (!username || !password) return
      const { clientId } = getAuthParamsFromUrl()
      authenticate({
        data: { username, password },
        realm: realm_name ?? 'master',
        clientId,
      })
    },
    [authenticate, getAuthParamsFromUrl, realm_name],
  )

  if (pageType === 'login') {
    return { onSubmit: loginSubmit }
  }

  return {}
}
