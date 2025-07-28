import { z } from 'zod'

export const verifyOtpSchema = z.object({
  pin: z.string().min(6, {
    message: 'Pin must be at least 6 characters long',
  }),
  deviceName: z.string().min(1, {
    message: 'Device name must be at least 1 character long',
  }),
})

export type VerifyOtpSchema = z.infer<typeof verifyOtpSchema>
