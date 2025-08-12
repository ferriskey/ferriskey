import { RequiredAction } from '@/api/core.interface'
import { match } from 'ts-pattern'
import ConfigureOtpFeature from '../feature/execution/configure-otp-feature'
import UpdatePasswordFeature from '@/pages/authentication/feature/execution/update-password-feature.tsx'

export interface PageRequiredActionProps {
  execution: string
}

export default function PageRequiredAction({ execution }: PageRequiredActionProps) {
  return match(execution.toLowerCase())
    .with(RequiredAction.ConfigureOtp, () => <ConfigureOtpFeature />)
    .with(RequiredAction.UpdatePassword, () => <UpdatePasswordFeature />)
    .otherwise(() => <div>No action required</div>)
}
