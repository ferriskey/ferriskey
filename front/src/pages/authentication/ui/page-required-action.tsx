import { RequiredAction } from '@/api/core.interface'
import { match } from 'ts-pattern'
import { useTranslation } from 'react-i18next'
import ConfigureOtpFeature from '../feature/execution/configure-otp-feature'
import UpdatePasswordFeature from '@/pages/authentication/feature/execution/update-password-feature.tsx'
import ConfigurePasskeyFeature from '../feature/execution/configure-passkey-feature'
import VerifyEmailFeature from '../feature/execution/verify-email-feature'
import { PortalLayoutWrapper } from '../components/portal-layout-wrapper'
import { AUTH_NAMESPACE, PORTAL_PAGE_TYPE } from '../constants'

export interface PageRequiredActionProps {
  execution: string
}

export default function PageRequiredAction({ execution }: PageRequiredActionProps) {
  const { t } = useTranslation(AUTH_NAMESPACE)

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
      <PortalLayoutWrapper pageType={PORTAL_PAGE_TYPE.TOTP_SETUP}>
        <ConfigureOtpFeature />
      </PortalLayoutWrapper>
    ))
    .with(RequiredAction.UpdatePassword, () => <UpdatePasswordFeature />)
    .with(RequiredAction.ConfigurePasskey, () => <ConfigurePasskeyFeature />)
    .with(RequiredAction.VerifyEmail, () => (
      <PortalLayoutWrapper pageType={PORTAL_PAGE_TYPE.VERIFY_EMAIL}>
        <VerifyEmailFeature />
      </PortalLayoutWrapper>
    ))
    .otherwise(() => <div>{t('required_action.none')}</div>)
}
