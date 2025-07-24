import { useSetupOtp } from '@/api/trident.api'
import ConfigureOtp from '../../ui/execution/configure-otp'
import { useParams, useSearchParams } from 'react-router'
import { RouterParams } from '@/routes/router'

export default function ConfigureOtpFeature() {
  const { realm_name } = useParams<RouterParams>()
  const [searchParams] = useSearchParams()
  const token = searchParams.get('client_data')

  const { data } = useSetupOtp({
    realm: realm_name ?? 'master',
    token: token,
  })

  return <ConfigureOtp qrCodeUrl={data?.otpauth_url} secret={data?.secret} />
}
