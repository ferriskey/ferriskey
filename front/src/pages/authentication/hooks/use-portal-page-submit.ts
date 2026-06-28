import { useCallback, useEffect, useRef, useState } from 'react'
import { useNavigate } from 'react-router'
import { toast } from 'sonner'
import { AuthenticationStatus } from '@/api/api.interface'
import {
  useAuthenticateMutation,
  useRegistrationMutation,
  useResendVerificationEmailMutation,
} from '@/api/auth.api'
import { useForgotPassword, useResetPassword } from '@/api/password-reset.api'
import { useSendMagicLink, useSetupOtp, useVerifyOtp } from '@/api/trident.api'
import { useDeviceVerify } from '@/api/device.api'
import type { Schemas } from '@/api/api.client'
import { useAuth } from '@/hooks/use-auth'
import { useOAuthParams } from './use-oauth-params'
import { POST_LOGIN_RETURN_KEY } from '../feature/page-device-verify-feature'

export type DeviceVerifyResult = 'approved' | 'denied' | null

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
  /**
   * Device-verify only: the deny action. Fired by the wrapper when a
   * `device_deny_button` (`data-fk-action="device-deny"`) is clicked —
   * reads the typed `user_code` from the form and POSTs with action=deny.
   */
  onDeviceDeny?: (data: FormData) => void
  /**
   * Device-verify only: `'approved'` / `'denied'` once the backend has
   * recorded the decision. The wrapper swaps the custom tree for the
   * hardcoded result screen when this is set; `null` while the form is
   * still being filled in.
   */
  deviceResult?: DeviceVerifyResult
  /** Device-verify only: clears the result so the user can verify another code. */
  onDeviceReset?: () => void
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

  // Forgot password: collect the email, POST to the realm's
  // `/login-actions/forgot-password`. The endpoint always returns 200
  // (anti-enumeration), and `useForgotPassword` already toasts both
  // success and failure — we only need to surface client-side empty-email
  // feedback via the form-error banner.
  const { mutate: forgotPassword, isPending: isForgotPasswordPending } =
    useForgotPassword()
  const forgotPasswordSubmit = useCallback(
    (data: FormData) => {
      const email = String(data.get('email') ?? '').trim()
      if (!email) {
        onFormError?.('Enter your email to receive a reset link.')
        return
      }
      onFormError?.(null)
      forgotPassword({
        path: { realm_name: realm_name ?? 'master' },
        body: { email },
      })
    },
    [forgotPassword, onFormError, realm_name],
  )

  // Reset password: the token is the `token_id` in the URL (set by the
  // backend's email link). Reads new password + confirm from the form,
  // verifies match, then POSTs the reset.
  const { mutate: resetPassword, isPending: isResetPasswordPending } =
    useResetPassword()
  const resetPasswordSubmit = useCallback(
    (data: FormData) => {
      const password = String(data.get('password') ?? '')
      const passwordConfirm = String(data.get('password_confirm') ?? '')
      if (!password) {
        onFormError?.('Choose a new password.')
        return
      }
      if (passwordConfirm && passwordConfirm !== password) {
        onFormError?.('Passwords do not match.')
        return
      }
      const params = new URLSearchParams(window.location.search)
      const tokenId = params.get('token_id') ?? params.get('token') ?? ''
      const token = params.get('token') ?? ''
      if (!tokenId) {
        onFormError?.('Reset link is missing or expired. Request a new one.')
        return
      }
      onFormError?.(null)
      resetPassword(
        {
          path: { realm_name: realm_name ?? 'master' },
          body: { token_id: tokenId, token, new_password: password },
        },
        {
          // The backend signs the user back in on a successful reset and
          // returns the OAuth callback URL (`login_url` — `code` + `state`)
          // — same shape as a normal login. Following it lets the app
          // exchange the code for tokens and land on the post-login
          // destination, which matches the React fallback's behaviour.
          // If for any reason `login_url` is missing, fall back to the
          // login page with a success toast so the user can re-sign-in
          // manually with their new password.
          onSuccess: (response) => {
            const loginUrl = (response as { login_url?: string | null } | undefined)
              ?.login_url
            if (loginUrl) {
              window.location.href = loginUrl
              return
            }
            toast.success('Password updated. Sign in with your new password.')
            navigate(`/realms/${realm_name ?? 'master'}/authentication/login`)
          },
          onError: (error: Error) =>
            onFormError?.(error.message || 'Could not reset password.'),
        },
      )
    },
    [navigate, onFormError, realm_name, resetPassword],
  )
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

  // Device verify (RFC 8628): approve is the form submit, deny is the
  // `data-fk-action="device-deny"` button routed back here by the wrapper.
  // Both read the typed `user_code` (the segmented input holds 8 flat chars,
  // so we re-insert the dash to match the stored XXXX-XXXX code) and POST to
  // the verify endpoint. A 401 means "not signed in yet" — stash the return
  // URL and bounce to login, exactly like the hardcoded device feature.
  const { mutateAsync: verifyDevice, isPending: isVerifyingDevice } = useDeviceVerify()
  const [deviceResult, setDeviceResult] = useState<DeviceVerifyResult>(null)
  const deviceRedirectingToLogin = useRef(false)
  const runDeviceVerify = useCallback(
    async (data: FormData, action: 'approve' | 'deny') => {
      const raw = String(data.get('user_code') ?? '')
        .trim()
        .toUpperCase()
        .replace('-', '')
      if (raw.length !== 8) {
        onFormError?.('Enter the 8-character code shown on your device.')
        return
      }
      const userCode = `${raw.slice(0, 4)}-${raw.slice(4)}`
      onFormError?.(null)
      try {
        const response = await verifyDevice({
          realm: realm_name ?? 'master',
          data: { user_code: userCode, action },
        })
        setDeviceResult(response.status === 'denied' ? 'denied' : 'approved')
      } catch (err) {
        const error = err as {
          status?: number
          data?: { error_description?: string; redirect_uri?: string }
          message?: string
        }
        if (error.status === 401) {
          if (deviceRedirectingToLogin.current) return
          deviceRedirectingToLogin.current = true
          try {
            sessionStorage.setItem(
              POST_LOGIN_RETURN_KEY,
              error.data?.redirect_uri ??
                `${window.location.pathname}${window.location.search}`,
            )
          } catch {
            // sessionStorage disabled — the user lands on /overview after
            // login instead of back here. Acceptable degradation.
          }
          navigate(`/realms/${realm_name ?? 'master'}/authentication/login`, {
            replace: true,
          })
          return
        }
        const description =
          error.data?.error_description ??
          error.message ??
          'Unable to verify this code. Please try again.'
        onFormError?.(description)
      }
    },
    [navigate, onFormError, realm_name, verifyDevice],
  )
  const deviceApproveSubmit = useCallback(
    (data: FormData) => {
      void runDeviceVerify(data, 'approve')
    },
    [runDeviceVerify],
  )
  const deviceDenySubmit = useCallback(
    (data: FormData) => {
      void runDeviceVerify(data, 'deny')
    },
    [runDeviceVerify],
  )
  const deviceReset = useCallback(() => {
    deviceRedirectingToLogin.current = false
    setDeviceResult(null)
    onFormError?.(null)
  }, [onFormError])

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
  if (pageType === 'forgot_password') {
    return { onSubmit: forgotPasswordSubmit, isSubmitting: isForgotPasswordPending }
  }
  if (pageType === 'reset_password') {
    return { onSubmit: resetPasswordSubmit, isSubmitting: isResetPasswordPending }
  }
  if (pageType === 'device_verify') {
    return {
      onSubmit: deviceApproveSubmit,
      onDeviceDeny: deviceDenySubmit,
      deviceResult,
      onDeviceReset: deviceReset,
      isSubmitting: isVerifyingDevice,
    }
  }

  return { isSubmitting: false }
}
