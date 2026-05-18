import { Route, Routes } from 'react-router-dom'
import PageLoginFeature from './feature/page-login-feature'
import PageCallbackFeature from './feature/page-callback-feature'
import PageRequiredActionFeature from './feature/page-required-action-feature'
import PageOtpChallengeFeature from './feature/page-otp-challenge-feature'
import PageRegisterFeature from './feature/page-register-feature'
import PageForgotPasswordFeature from './feature/page-forgot-password-feature'
import PageResetPasswordFeature from './feature/page-reset-password-feature'
import PageMagicLinkVerifyFeature from './feature/page-magic-link-verify-feature'
import PageCheckYourEmail from './feature/page-check-your-email'
import PageVerifyEmail from './feature/page-verify-email'
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
          <Portal pageType='verify_email'>
            <PageVerifyEmail />
          </Portal>
        }
      />
      {/* Routes without a portal page type render bare. */}
      <Route path='/callback' element={<PageCallbackFeature />} />
      <Route path='/required-action' element={<PageRequiredActionFeature />} />
      <Route path='/check-your-email' element={<PageCheckYourEmail />} />
    </Routes>
  )
}
