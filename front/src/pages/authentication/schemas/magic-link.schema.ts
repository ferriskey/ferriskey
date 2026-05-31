import { z } from 'zod'

export const magicLinkSchema = z.object({
  email: z
    .string()
    .min(1, { message: 'Email is required' })
    .email({ message: 'Please enter a valid email address' }),
})

export type MagicLinkSchema = z.infer<typeof magicLinkSchema>
