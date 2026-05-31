import { useCallback, useEffect } from 'react'
import { useNavigate } from 'react-router'
import { toast } from 'sonner'
import { AuthenticationStatus } from '@/api/api.interface'
import {
  useAuthenticateMutation,
  useRegistrationMutation,
  useResendVerificationEmailMutation,
} from '@/api/auth.api'
import { useSendMagicLink, useSetupOtp, useVerifyOtp } from '@/api/trident.api'
import type { Schemas } from '@/api/api.client'
import { useAuth } from '@/hooks/use-auth'
import { useOAuthParams } from './use-oauth-params'

/**
 * Returns an `onSubmit(FormData)` handler tailored to the portal page type.
 * The wrapper passes this to the renderer when the realm admin has built a
 * custom page tree; the renderer wraps the tree in a <form> and the submit
 * block triggers this handler with the inputs' name-keyed values.
 *
 * `onFormError` is invoked whenever a submit fails with a user-meaningful
 * message — the wrapper hoists this into a `form_error_banner` block so
 * the page can show the error inline (mirrors what the React default
 * theme does with a red banner above the form). When the handler omits
 * the callback or the user fixes the field, callers can pass `null` to
 * clear the banner.
 */
export function usePortalPageSubmit(
  pageType: Schemas.PortalPageType,
  options?: { onFormError?: (message: string | null) => void },
): {
  onSubmit?: (data: FormData) => void
  /**
   * `true` while the current page's submit network call is in flight.
   * The wrapper uses this to flip the `submit_button` block into its
   * loader/disabled state — so the user gets a visible "click did
   * something" hint instead of staring at a frozen form.
   */
  isSubmitting: boolean
} {
  const onFormError = options?.onFormError
  const { realm_name, getAuthParamsFromUrl } = useOAuthParams()
  const navigate = useNavigate()
  const {
    mutate: authenticate,
    data: authenticateData,
    isPending: isAuthenticating,
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
      if (!username || !password) {
        const msg = 'Enter your username and password to continue.'
        onFormError?.(msg)
        return
      }
      // Clear any stale error from a previous attempt before firing the
      // new one — keeps the banner from showing a misleading message
      // while the network call is in flight.
      onFormError?.(null)
      const { clientId } = getAuthParamsFromUrl()
      authenticate(
        {
          data: { username, password },
          realm: realm_name ?? 'master',
          clientId,
        },
        {
          // Surface the API error so the page can render its
          // `form_error_banner` block inline (mirrors the React default
          // theme's red banner). For "Session expired" the mutation
          // itself toasts with a Reload action — we don't double up.
          onError: (error: Error) => {
            if (/session\s+expired/i.test(error.message)) return
            onFormError?.(error.message || 'Sign-in failed. Please try again.')
          },
        },
      )
    },
    [authenticate, getAuthParamsFromUrl, onFormError, realm_name],
  )

  // Magic-link request: the customised page collects an email and hands it
  // straight to the same `send-magic-link` endpoint the hardcoded
  // `PageMagicLinkRequestFeature` uses. We don't navigate on success —
  // staying on the same page lets the admin's tree render its own "we sent
  // it" state if they've authored one; a toast keeps the user informed
  // either way as a safety net.
  const { mutate: sendMagicLink, isPending: isSendingMagicLink } = useSendMagicLink()
  const magicLinkSubmit = useCallback(
    (data: FormData) => {
      const email = String(data.get('email') ?? data.get('username') ?? '').trim()
      if (!email) return
      sendMagicLink(
        {
          path: { realm_name: realm_name ?? 'master' },
          body: { email },
        },
        {
          onSuccess: () => toast.success('Magic link sent — check your inbox.'),
          onError: () => toast.error('Failed to send magic link'),
        },
      )
    },
    [realm_name, sendMagicLink],
  )

  // Register: mirrors the React `PageRegisterFeature` payload — email,
  // password, optional first/last name and username. Confirm-password (if
  // the admin includes it) is checked client-side before the POST; an empty
  // confirm field passes (treated as "no confirmation requested").
  const {
    mutate: registration,
    data: registrationData,
    isPending: isRegistering,
  } = useRegistrationMutation()
  const { setAuthTokens } = useAuth()
  useEffect(() => {
    if (pageType !== 'register' || !registrationData) return
    if (registrationData.status === 'redirect') {
      window.location.href = registrationData.data.url
      return
    }
    if (registrationData.status === 'authenticated') {
      setAuthTokens(
        registrationData.data.access_token,
        registrationData.data.refresh_token,
        registrationData.data.id_token ?? null,
      )
      navigate(`/realms/${realm_name}/overview`, { replace: true })
      return
    }
    // Backend returned a "verify your email" state — go to the confirmation
    // screen. The email is carried via location state purely for display.
    navigate(`/realms/${realm_name}/authentication/check-your-email`, {
      replace: true,
    })
  }, [pageType, registrationData, setAuthTokens, navigate, realm_name])

  const registerSubmit = useCallback(
    (data: FormData) => {
      const email = String(data.get('email') ?? '').trim()
      const password = String(data.get('password') ?? '')
      const passwordConfirm = String(data.get('password_confirm') ?? '')
      if (passwordConfirm && passwordConfirm !== password) {
        toast.error('Passwords do not match.')
        return
      }
      if (!email || !password) {
        toast.error('Email and password are required.')
        return
      }
      registration({
        path: { realm_name: realm_name ?? 'master' },
        body: {
          email,
          password,
          // Username defaults to the local-part of the email so a tree that
          // omits the field still produces a valid record — admins who want
          // explicit usernames just add a `username_input` block.
          username: String(data.get('username') ?? '').trim() || email.split('@')[0],
          first_name: String(data.get('first_name') ?? '').trim() || null,
          last_name: String(data.get('last_name') ?? '').trim() || null,
        },
      })
    },
    [realm_name, registration],
  )

  // Verify-email: the only meaningful action from the user is "send me
  // another verification email". The token that authenticates the resend
  // call lives in the URL as `client_data` (the React fallback stores it
  // in sessionStorage too, but the portal tree submit has no access to
  // that — reading from the URL is the canonical source).
  const { mutate: resendVerification, isPending: isResendingVerification } =
    useResendVerificationEmailMutation()
  const verifyEmailSubmit = useCallback(
    () => {
      const token = new URLSearchParams(window.location.search).get('client_data')
      if (!token) {
        toast.error('Cannot resend verification email — missing authentication context.')
        return
      }
      resendVerification(
        { realm: realm_name ?? 'master', token },
        {
          onSuccess: () => toast.success('Verification email sent. Check your inbox.'),
          onError: (error) => toast.error(error.message || 'Failed to resend verification email'),
        },
      )
    },
    [realm_name, resendVerification],
  )

  // TOTP setup: submit reads the 6-digit code + optional device label from
  // the form, pulls the `secret` from the cached `useSetupOtp` query
  // (already prefetched by `PortalLayoutWrapper`), and posts to verify.
  // On success the backend redirects via `authenticate({ useToken: true })`
  // — same chained pattern as the React `ConfigureOtpFeature`.
  //
  // All hooks for this branch run unconditionally (rules-of-hooks); the
  // query just sits idle when we're not on the totp_setup page because we
  // pass `token: null` and the API hook short-circuits on falsy token.
  const totpSetupToken =
    typeof window !== 'undefined'
      ? new URLSearchParams(window.location.search).get('client_data')
      : null
  const { data: setupData } = useSetupOtp({
    realm: realm_name ?? 'master',
    token: pageType === 'totp_setup' ? totpSetupToken : null,
  })
  const {
    mutate: verifyOtp,
    data: verifyOtpData,
    status: verifyOtpStatus,
    isPending: isVerifyingOtp,
  } = useVerifyOtp()
  // After a successful verify, hand control back to the auth flow so the
  // next required action (or final redirect) runs.
  const { mutate: authenticateAfterTotp, data: authenticateAfterTotpData } =
    useAuthenticateMutation()
  useEffect(() => {
    if (pageType !== 'totp_setup') return
    if (verifyOtpData && verifyOtpStatus === 'success' && totpSetupToken) {
      authenticateAfterTotp({
        clientId: 'security-admin-console',
        realm: realm_name ?? 'master',
        data: {},
        useToken: true,
        token: totpSetupToken,
      })
    }
  }, [
    pageType,
    verifyOtpData,
    verifyOtpStatus,
    totpSetupToken,
    authenticateAfterTotp,
    realm_name,
  ])
  useEffect(() => {
    if (pageType !== 'totp_setup' || !authenticateAfterTotpData) return
    if (authenticateAfterTotpData.url) {
      window.location.href = authenticateAfterTotpData.url
      return
    }
    if (
      authenticateAfterTotpData.status === AuthenticationStatus.RequiresActions &&
      authenticateAfterTotpData.required_actions &&
      authenticateAfterTotpData.required_actions.length > 0 &&
      authenticateAfterTotpData.token
    ) {
      const first = authenticateAfterTotpData.required_actions[0]
      navigate(
        `/realms/${realm_name}/authentication/required-action?execution=${first.toUpperCase()}&client_data=${authenticateAfterTotpData.token}`,
      )
    }
  }, [pageType, authenticateAfterTotpData, navigate, realm_name])

  const totpSetupSubmit = useCallback(
    (data: FormData) => {
      const code = String(data.get('totp') ?? '').trim()
      const label = String(data.get('device_name') ?? '').trim() || 'Authenticator'
      if (!code) {
        toast.error('Enter the 6-digit code from your authenticator app.')
        return
      }
      const secret = setupData?.secret
      if (!totpSetupToken || !secret) {
        toast.error('TOTP setup context is missing. Please restart the flow.')
        return
      }
      verifyOtp(
        {
          data: { code, label, secret },
          token: totpSetupToken,
          realm: realm_name,
        },
        {
          onError: () =>
            toast.error('Code didn\u2019t match — try the latest one from your app.'),
        },
      )
    },
    [realm_name, setupData, totpSetupToken, verifyOtp],
  )

  // All hooks above run unconditionally so the order stays stable across
  // renders. We only branch on `pageType` *here*, at the return level, to
  // pick which submit handler the wrapper exposes for this page. The
  // `isSubmitting` flag aggregates the in-flight mutations that matter for
  // *this* page, so the `submit_button` block knows when to show its
  // loader without leaking unrelated busy states (e.g. a magic-link send
  // doesn't affect the totp-setup page's submit button).
  if (pageType === 'login') {
    return { onSubmit: loginSubmit, isSubmitting: isAuthenticating }
  }
  if (pageType === 'magic_link_request') {
    return { onSubmit: magicLinkSubmit, isSubmitting: isSendingMagicLink }
  }
  if (pageType === 'register') {
    return { onSubmit: registerSubmit, isSubmitting: isRegistering }
  }
  if (pageType === 'verify_email') {
    return { onSubmit: verifyEmailSubmit, isSubmitting: isResendingVerification }
  }
  if (pageType === 'totp_setup') {
    return { onSubmit: totpSetupSubmit, isSubmitting: isVerifyingOtp }
  }

  return { isSubmitting: false }
}
