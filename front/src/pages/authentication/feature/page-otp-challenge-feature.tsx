import { useLocation, useNavigate, useParams } from 'react-router'
import PageOtpChallenge from '../ui/page-otp-challenge'
import { RouterParams } from '@/routes/router'
import { useChallengeOtp } from '@/api/trident.api'
import { useForm } from 'react-hook-form'
import { challengeOtpSchema, ChallengeOtpSchema } from '../schemas/challange-otp.schema'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form } from '@/components/ui/form'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { AUTH_NAMESPACE } from '../constants'

export default function PageOtpChallengeFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const { state } = useLocation()
  const navigate = useNavigate()
  const { mutate: challengeOtp, data: challengeOtpData, isPending, error } = useChallengeOtp()

  const email = (state as { email?: string | null } | null)?.email ?? ''

  const form = useForm<ChallengeOtpSchema>({
    resolver: zodResolver(challengeOtpSchema),
    defaultValues: {
      code: '',
    },
  })

  const handleCancelClick = () => {
    navigate(`/realms/${realm_name}/authentication/login`)
  }

  const handleClick = (values: ChallengeOtpSchema) => {
    challengeOtp({
      data: {
        code: values.code,
      },
      realm: realm_name,
    })
  }

  useEffect(() => {
    if (!challengeOtpData) return

    if (challengeOtpData.required_actions && challengeOtpData.required_actions.length > 0) {
      const firstRequiredAction = challengeOtpData.required_actions[0]
      navigate(
        `/realms/${realm_name}/authentication/required-action?execution=${firstRequiredAction.toUpperCase()}`
      )
      return
    }

    if (challengeOtpData.url) {
      window.location.href = challengeOtpData.url
    }
  }, [challengeOtpData, navigate, realm_name])

  useEffect(() => {
    if (error) {
      toast.error(t('otp.invalid'))
    }
  }, [error, t])

  return (
    <Form {...form}>
      <PageOtpChallenge
        handleCancelClick={handleCancelClick}
        handleClick={handleClick}
        email={email}
        isLoading={isPending}
      />
    </Form>
  )
}
