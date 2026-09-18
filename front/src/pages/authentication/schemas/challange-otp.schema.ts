import { z } from 'zod'
import { translate } from '@/lib/i18n'
import { AUTH_NAMESPACE } from '../constants'

export const challengeOtpSchema = z.object({
  code: z.string().min(6, {
    error: () => translate(`${AUTH_NAMESPACE}:validation.otp_code_length`),
  }),
})

export type ChallengeOtpSchema = z.infer<typeof challengeOtpSchema>
