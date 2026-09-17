import { z } from 'zod'
import { translate } from '@/lib/i18n'

const emailField = z
  .union([
    z.string().email({ error: () => translate('user:validation.email_invalid') }),
    z.literal(''),
  ])
  .optional()

export const createUserValidator = z.object({
  username: z.string().min(1, { error: () => translate('user:validation.username_required') }),
  firstname: z.string().optional(),
  lastname: z.string().optional(),
  email: emailField,
  email_verified: z.boolean().optional(),
})

export const updateUserValidator = z.object({
  username: z.string().min(1, { error: () => translate('user:validation.username_required') }),
  firstname: z.string().optional(),
  lastname: z.string().optional(),
  enabled: z.boolean().optional(),
  email: emailField,
  email_verified: z.boolean().optional(),
  required_actions: z.array(z.string()).optional(),
})


export type CreateUserSchema = z.infer<typeof createUserValidator>
export type UpdateUserSchema = z.infer<typeof updateUserValidator>
