import { useNavigate, useParams, useSearchParams } from 'react-router'
import PageOtpChallenge from '../ui/page-otp-challenge'
import { RouterParams } from '@/routes/router'
import { useChallengeOtp } from '@/api/trident.api'
import { useForm } from 'react-hook-form'
import { challengeOtpSchema, ChallengeOtpSchema } from '../schemas/challange-otp.schema'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form } from '@/components/ui/form'
import { useCallback, useEffect } from 'react'

export default function PageOtpChallengeFeature() {
  const { realm_name } = useParams<RouterParams>()
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()
  const { mutate: challengeOtp, data: challengeOtpData } = useChallengeOtp()

  const token = searchParams.get('token')

  const email = useCallback(() => {
    // the token is a JWT we need to decode it to get the claim "email"
    if (!token) return ''
    const decodedToken = JSON.parse(atob(token.split('.')[1]))
    return decodedToken.email
  }, [token])

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
    if (!token) return
    challengeOtp({
      data: {
        code: values.code,
      },
      token,
      realm: realm_name,
    })
  }

  useEffect(() => {
    if (!challengeOtpData) return

    window.location.href = challengeOtpData.url
  }, [challengeOtpData])

  return (
    <Form {...form}>
      <PageOtpChallenge
        handleCancelClick={handleCancelClick}
        handleClick={handleClick}
        email={email()}
      />
    </Form>
  )
}
