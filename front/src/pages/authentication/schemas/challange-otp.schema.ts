import { z } from 'zod'

export const challengeOtpSchema = z.object({
  code: z.string().min(6, {
    message: 'Code must be at least 6 characters long',
  }),
})

export type ChallengeOtpSchema = z.infer<typeof challengeOtpSchema>
