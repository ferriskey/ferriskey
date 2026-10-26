import { useSetupOtp, useVerifyOtp } from '@/api/trident.api'
import ConfigureOtp from '../../ui/execution/configure-otp'
import { useNavigate, useParams } from 'react-router'
import type { RouterParams } from '@/routes/router'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import type { VerifyOtpSchema } from '../../schemas/verify-otp.schema'
import { verifyOtpSchema } from '../../schemas/verify-otp.schema'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form } from '@/components/ui/form'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { AUTH_NAMESPACE } from '../../constants'
import { continueLogin } from '../../utils/continue-login'

export default function ConfigureOtpFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const navigate = useNavigate()
  const { mutate: verifyOtp, data: verifyOtpData, status: verifyOtpStatus } = useVerifyOtp()

  const { data, isError, error } = useSetupOtp({
    realm: realm_name ?? 'master',
  })

  useEffect(() => {
    if (isError) {
      toast.error(t('configure_otp.setup_failed'))
      console.error(error)
    }
  }, [isError, error, t])

  const form = useForm<VerifyOtpSchema>({
    resolver: zodResolver(verifyOtpSchema),
    defaultValues: {
      pin: '',
      deviceName: '',
    },
  })

  const handleSubmit = (values: VerifyOtpSchema) => {
    if (!data) {
      toast.error(t('configure_otp.not_ready'))
      return
    }

    // The secret is never sent back: the server verifies the code against the
    // enrollment it recorded when it issued the secret.
    verifyOtp({
      data: {
        code: values.pin,
        label: values.deviceName,
      },
      realm: realm_name,
    })
  }

  useEffect(() => {
    if (verifyOtpData && verifyOtpStatus === 'success') {
      continueLogin(verifyOtpData, navigate, realm_name)
    }
  }, [verifyOtpData, verifyOtpStatus, navigate, realm_name])

  return (
    <Form {...form}>
      <ConfigureOtp
        handleSubmit={handleSubmit}
        qrCodeUrl={data?.otpauth_url}
        secret={data?.secret}
      />
    </Form>
  )
}
