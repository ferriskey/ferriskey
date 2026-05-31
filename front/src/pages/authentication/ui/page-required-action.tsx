import { RequiredAction } from '@/api/core.interface'
import { match } from 'ts-pattern'
import ConfigureOtpFeature from '../feature/execution/configure-otp-feature'
import UpdatePasswordFeature from '@/pages/authentication/feature/execution/update-password-feature.tsx'
import ConfigurePasskeyFeature from '../feature/execution/configure-passkey-feature'
import VerifyEmailFeature from '../feature/execution/verify-email-feature'
import { PortalLayoutWrapper } from '../components/portal-layout-wrapper'

export interface PageRequiredActionProps {
  execution: string
}

export default function PageRequiredAction({ execution }: PageRequiredActionProps) {
  // Wrap executions that have a matching `PortalPageType` in
  // `<PortalLayoutWrapper>` so the realm admin's custom theme tree is
  // applied during the actual auth flow — not just on the direct
  // `/verify-email` route. Without this, a user finishing login with a
  // `VERIFY_EMAIL` required action lands on the bare React fallback even
  // when a custom verify-email tree exists on the active theme.
  //
  // `update_password` could map to `reset_password` but the semantics
  // differ (forced rotation vs. forgot-link flow); keep it bare until we
  // add a dedicated portal page type. `configure_otp` / `configure_passkey`
  // similarly have no portal page type yet.
  return match(execution.toLowerCase())
    .with(RequiredAction.ConfigureOtp, () => (
      <PortalLayoutWrapper pageType='totp_setup'>
        <ConfigureOtpFeature />
      </PortalLayoutWrapper>
    ))
    .with(RequiredAction.UpdatePassword, () => <UpdatePasswordFeature />)
    .with(RequiredAction.ConfigurePasskey, () => <ConfigurePasskeyFeature />)
    .with(RequiredAction.VerifyEmail, () => (
      <PortalLayoutWrapper pageType='verify_email'>
        <VerifyEmailFeature />
      </PortalLayoutWrapper>
    ))
    .otherwise(() => <div>No action required</div>)
}
