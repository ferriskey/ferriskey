import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const updateOwnProfileValidator = z.object({
  username: z.string().min(1, { error: () => translate('account:validation.username_required') }),
  firstname: z.string().optional(),
  lastname: z.string().optional(),
  email: z
    .union([
      z.string().email({ error: () => translate('account:validation.email_invalid') }),
      z.literal(''),
    ])
    .optional(),
})

export type UpdateOwnProfileSchema = z.infer<typeof updateOwnProfileValidator>
