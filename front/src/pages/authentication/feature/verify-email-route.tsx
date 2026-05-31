import { useEffect, useRef } from 'react'
import { useNavigate } from 'react-router'
import { useAuthenticateMutation } from '@/api/auth.api'
import { AuthenticationStatus } from '@/api/api.interface'
import { useVerifyEmail } from '@/hooks/use-verify-email'
import { PortalLayoutWrapper } from '../components/portal-layout-wrapper'
import PageVerifyEmail from './page-verify-email'
import {
  clearVerifyEmailContext,
  getVerifyEmailContext,
} from './execution/verify-email-feature'

/**
 * Orchestrates the email-verification flow for `/verify-email?token=…`:
 *
 *  1. Calls the verify endpoint via `useVerifyEmail` (state machine:
 *     loading / success / expired / error).
 *  2. On success, optionally chains an `authenticate({ useToken: true })`
 *     if the user still has a verify-email context cookie (registration
 *     was on this same browser session) — that lets the backend resume
 *     the auth flow without bouncing through login.
 *  3. Once everything resolves, redirects to `/email-verified` (a
 *     customisable success page) so the user never sits indefinitely on
 *     the verification screen.
 *
 * Crucially, this orchestration lives *above* `<PortalLayoutWrapper>`.
 * Without that hoist, a custom verify-email tree built in the theme
 * builder would shadow `<PageVerifyEmail>` and the verify mutation
 * would never run — the user would be stuck on the page even after
 * the link had been clicked.
 */
export default function VerifyEmailRoute() {
  const { state, realm_name } = useVerifyEmail()
  const navigate = useNavigate()
  const {
    mutate: authenticate,
    data: authenticateData,
  } = useAuthenticateMutation()
  const hasTriedAuth = useRef(false)
  const hasNavigatedToSuccess = useRef(false)

  const goToSuccess = () => {
    if (hasNavigatedToSuccess.current) return
    hasNavigatedToSuccess.current = true
    navigate(`/realms/${realm_name}/authentication/email-verified`, {
      replace: true,
    })
  }

  // Verification just succeeded — try to chain an auth call if we still
  // have the registration context. Without context, jump straight to the
  // success screen so the user isn't stuck on a "verifying…" message.
  useEffect(() => {
    if (state !== 'success' || hasTriedAuth.current) return

    const context = getVerifyEmailContext()
    if (!context || context.realm !== realm_name) {
      goToSuccess()
      return
    }

    hasTriedAuth.current = true
    authenticate({
      clientId: context.clientId,
      realm: context.realm,
      data: {},
      useToken: true,
      token: context.token,
    })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [state, realm_name, authenticate])

  // Authentication result handler. If the backend hands back a redirect
  // URL or a follow-up required action, we honour it. Otherwise (most
  // common case — verification just confirmed the email, no further
  // step needed), drop the user on `/email-verified`.
  useEffect(() => {
    if (!authenticateData) return

    clearVerifyEmailContext()

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
      window.location.href = `/realms/${realm_name}/authentication/required-action?execution=${first.toUpperCase()}&client_data=${authenticateData.token}`
      return
    }

    goToSuccess()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [authenticateData, realm_name])

  // Visual: the user's custom verify_email tree (if any) renders inside
  // Portal; otherwise `<PageVerifyEmail>` fallback shows the loading /
  // expired / error states. Either way the orchestration above is what
  // drives the actual flow.
  return (
    <PortalLayoutWrapper pageType='verify_email'>
      <PageVerifyEmail state={state} />
    </PortalLayoutWrapper>
  )
}
