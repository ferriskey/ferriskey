import { z } from 'zod'

export const magicLinkSchema = z.object({
  email: z.string().email({ message: 'Please enter a valid email address' }),
})

export type MagicLinkSchema = z.infer<typeof magicLinkSchema>
