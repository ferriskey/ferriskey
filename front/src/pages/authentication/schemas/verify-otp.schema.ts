import { z } from 'zod'

export const verifyOtpSchema = z.object({
  pin: z.string().min(6, {
    message: 'Pin must be at least 6 characters long',
  }),
})

export type VerifyOtpSchema = z.infer<typeof verifyOtpSchema>
