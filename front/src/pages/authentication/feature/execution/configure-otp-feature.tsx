import { useSetupOtp, useVerifyOtp } from '@/api/trident.api'
import ConfigureOtp from '../../ui/execution/configure-otp'
import { useNavigate, useParams, useSearchParams } from 'react-router'
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
  const [searchParams] = useSearchParams()
  const {
    mutate: authenticate,
    data: authenticateData,
  } = useAuthenticateMutation()
  const navigate = useNavigate()
  const { mutate: verifyOtp, data: verifyOtpData, status: verifyOtpStatus } = useVerifyOtp()

  const token = searchParams.get('client_data')

  const { data } = useSetupOtp({
    realm: realm_name ?? 'master',
    token: token,
  })

  const form = useForm<VerifyOtpSchema>({
    resolver: zodResolver(verifyOtpSchema),
    defaultValues: {
      pin: '',
      deviceName: '',
    },
  })


  const handle = useCallback(() => {
    const cookies = document.cookie.split(';').reduce(
      (acc, cookie) => {
        const [key, value] = cookie.trim().split('=')
        acc[key] = value
        return acc
      },
      {} as Record<string, string>
    )

    const sessionCode = cookies['FERRISKEY_SESSION'] || ''

    authenticate({
      clientId: 'security-admin-console',
      realm: realm_name ?? 'master',
      data: {},
      sessionCode,
      useToken: true,
      token: token ?? undefined,
    })
  }, [authenticate, realm_name, token])

  const handleSubmit = (values: VerifyOtpSchema) => {
    if (!token || !data) {
      toast.error('Token is missing')
      return
    }

    verifyOtp({
      data: {
        code: values.pin,
        label: values.deviceName,
        secret: data.secret,
      },
      token,
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
      authenticateData.required_actions.length > 0 &&
      authenticateData.token
    ) {
      const firstRequiredAction = authenticateData.required_actions[0]

      navigate(
        `/realms/${realm_name}/authentication/required-action?execution=${firstRequiredAction.toUpperCase()}&client_data=${authenticateData.token}`
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
