import { z } from 'zod'
import { translate } from '@/lib/i18n'
import { AUTH_NAMESPACE } from '../constants'

export const forgotPasswordSchema = z.object({
  email: z.string().email({
    error: () => translate(`${AUTH_NAMESPACE}:validation.email_invalid_hint`),
  }),
})

export type ForgotPasswordSchema = z.infer<typeof forgotPasswordSchema>
