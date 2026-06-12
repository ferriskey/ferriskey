import { Route, Routes } from 'react-router-dom'
import PageLoginFeature from './feature/page-login-feature'
import PageCallbackFeature from './feature/page-callback-feature'
import PageRequiredActionFeature from './feature/page-required-action-feature'
import PageOtpChallengeFeature from './feature/page-otp-challenge-feature'
import PageRegisterFeature from './feature/page-register-feature'
import PageForgotPasswordFeature from './feature/page-forgot-password-feature'
import PageResetPasswordFeature from './feature/page-reset-password-feature'
import PageMagicLinkVerifyFeature from './feature/page-magic-link-verify-feature'
import PageMagicLinkRequestFeature from './feature/page-magic-link-request-feature'
import PageEmailVerifiedFeature from './feature/page-email-verified-feature'
import PageCheckYourEmail from './feature/page-check-your-email'
import PageDeviceVerifyFeature from './feature/page-device-verify-feature'
import VerifyEmailRoute from './feature/verify-email-route'
import { PortalLayoutWrapper } from './components/portal-layout-wrapper'
import type { Schemas } from '@/api/api.client'
import type { ReactNode } from 'react'

function Portal({
  pageType,
  children,
}: {
  pageType: Schemas.PortalPageType
  children: ReactNode
}) {
  return <PortalLayoutWrapper pageType={pageType}>{children}</PortalLayoutWrapper>
}

export default function PageAuthentication() {
  return (
    <Routes>
      <Route
        path='/login'
        element={
          <Portal pageType='login'>
            <PageLoginFeature />
          </Portal>
        }
      />
      <Route
        path='/register'
        element={
          <Portal pageType='register'>
            <PageRegisterFeature />
          </Portal>
        }
      />
      <Route
        path='/otp'
        element={
          <Portal pageType='totp'>
            <PageOtpChallengeFeature />
          </Portal>
        }
      />
      <Route
        path='/forgot-password'
        element={
          <Portal pageType='forgot_password'>
            <PageForgotPasswordFeature />
          </Portal>
        }
      />
      <Route
        path='/reset-password'
        element={
          <Portal pageType='reset_password'>
            <PageResetPasswordFeature />
          </Portal>
        }
      />
      <Route
        path='/magic-link'
        element={
          <Portal pageType='magic_link_verify'>
            <PageMagicLinkVerifyFeature />
          </Portal>
        }
      />
      <Route
        path='/verify-email'
        element={
          // The verification orchestration (call the verify endpoint,
          // chain auth, redirect to /email-verified on success) lives in
          // a wrapper *above* Portal so it runs even when the admin has
          // a custom `verify_email` tree — otherwise that custom tree
          // shadows `<PageVerifyEmail>` and the verify call never fires,
          // leaving the user stuck on the page after clicking the link.
          <VerifyEmailRoute />
        }
      />
      {/* Post-verification success screen. `/verify-email` redirects here
          once the backend confirms the click — keeps the verification
          state-machine OUT of the customisable tree so the admin's
          `email_verified` design isn't bypassing the actual confirmation
          call. */}
      <Route
        path='/email-verified'
        element={
          <Portal pageType='email_verified'>
            <PageEmailVerifiedFeature />
          </Portal>
        }
      />
      {/* `/magic-link-request` is the dedicated form a user lands on when
          clicking the `data-fk-action="magic-link"` button in a custom portal
          tree. Wrapped in <Portal /> so realm admins can customise its layout
          via the theme builder; the React feature acts as the fallback when
          no valid custom tree exists. */}
      <Route
        path='/magic-link-request'
        element={
          <Portal pageType='magic_link_request'>
            <PageMagicLinkRequestFeature />
          </Portal>
        }
      />
      {/* Routes without a portal page type render bare. */}
      <Route path='/callback' element={<PageCallbackFeature />} />
      <Route path='/required-action' element={<PageRequiredActionFeature />} />
      {/* RFC 8628 §3.3 device verification page. Rendered bare — no
          PortalPageType variant yet, theme builder support is a follow-up. */}
      <Route path='/device' element={<PageDeviceVerifyFeature />} />
      {/* "Check your inbox" screen reached right after registration — uses
          the `verify_email` portal pageType so the admin's customised
          design (heading, layout, link styling, theme tokens) applies
          to both the post-registration prompt and the post-link-click
          state. Same flow, same visual language. */}
      <Route
        path='/check-your-email'
        element={
          <Portal pageType='verify_email'>
            <PageCheckYourEmail />
          </Portal>
        }
      />
    </Routes>
  )
}
