import { RequiredAction } from '@/api/core.interface'
import { match } from 'ts-pattern'
import ConfigureOtpFeature from '../feature/execution/configure-otp-feature'
import UpdatePasswordFeature from '@/pages/authentication/feature/execution/update-password-feature.tsx'
import ConfigurePasskeyFeature from '../feature/execution/configure-passkey-feature'

export interface PageRequiredActionProps {
  execution: string
}

export default function PageRequiredAction({ execution }: PageRequiredActionProps) {
  return match(execution.toLowerCase())
    .with(RequiredAction.ConfigureOtp, () => <ConfigureOtpFeature />)
    .with(RequiredAction.UpdatePassword, () => <UpdatePasswordFeature />)
    .with(RequiredAction.ConfigurePasskey, () => <ConfigurePasskeyFeature />)
    .otherwise(() => <div>No action required</div>)
}
