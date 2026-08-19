import { useSetupOtp, useVerifyOtp } from '@/api/trident.api'
import ConfigureOtp from '../../ui/execution/configure-otp'
import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useAuthenticateMutation } from '@/api/auth.api'
import { useCallback, useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { verifyOtpSchema, VerifyOtpSchema } from '../../schemas/verify-otp.schema'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form } from '@/components/ui/form'
import { toast } from 'sonner'
import { AuthenticationStatus } from '@/api/api.interface'

export default function ConfigureOtpFeature() {
  const { realm_name } = useParams<RouterParams>()
  const {
    mutate: authenticate,
    data: authenticateData,
  } = useAuthenticateMutation()
  const navigate = useNavigate()
  const { mutate: verifyOtp, data: verifyOtpData, status: verifyOtpStatus } = useVerifyOtp()

  const { data, isError, error } = useSetupOtp({
    realm: realm_name ?? 'master',
  })

  useEffect(() => {
    if (isError) {
      toast.error('Error during OTP configuration')
      console.error(error)
    }
  }, [isError, error])

  const form = useForm<VerifyOtpSchema>({
    resolver: zodResolver(verifyOtpSchema),
    defaultValues: {
      pin: '',
    },
  })

  const handle = useCallback(() => {
    authenticate({
      clientId: 'security-admin-console',
      realm: realm_name ?? 'master',
      data: {},
    })
  }, [authenticate, realm_name])

  const handleSubmit = (values: VerifyOtpSchema) => {
    if (!data) {
      toast.error('OTP setup is not ready yet')
      return
    }

    // The secret is never sent back: the server verifies the code against the
    // enrollment it recorded when it issued the secret.
    verifyOtp({
      data: {
        code: values.pin,
      },
      realm: realm_name,
    })
  }

  useEffect(() => {
    if (verifyOtpData && verifyOtpStatus === 'success') {
      handle()
    }
  }, [verifyOtpData, handle, verifyOtpStatus])

  useEffect(() => {
    if (!authenticateData) return
    if (authenticateData.url) {
      window.location.href = authenticateData.url
    }

    if (
      authenticateData.status === AuthenticationStatus.RequiresActions &&
      authenticateData.required_actions &&
      authenticateData.required_actions.length > 0
    ) {
      const firstRequiredAction = authenticateData.required_actions[0]

      navigate(
        `/realms/${realm_name}/authentication/required-action?execution=${firstRequiredAction.toUpperCase()}`
      )
    }
  }, [authenticateData, navigate, realm_name])

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
