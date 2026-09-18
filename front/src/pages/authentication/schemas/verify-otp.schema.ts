import { z } from 'zod'
import { translate } from '@/lib/i18n'
import { AUTH_NAMESPACE } from '../constants'

export const verifyOtpSchema = z.object({
  pin: z.string().min(6, {
    error: () => translate(`${AUTH_NAMESPACE}:validation.otp_pin_length`),
  }),
  deviceName: z.string().min(1, {
    error: () => translate(`${AUTH_NAMESPACE}:validation.device_name_required`),
  }),
})

export type VerifyOtpSchema = z.infer<typeof verifyOtpSchema>
