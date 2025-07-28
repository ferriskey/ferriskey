import { RequiredAction } from '@/api/api.interface'
import { match } from 'ts-pattern'
import ConfigureOtpFeature from '../feature/execution/configure-otp-feature'

export interface PageRequiredActionProps {
  execution: string
}

export default function PageRequiredAction({ execution }: PageRequiredActionProps) {
  return match(execution.toLowerCase())
    .with(RequiredAction.ConfigureOtp, () => <ConfigureOtpFeature />)
    .otherwise(() => <div>No action required</div>)
}
