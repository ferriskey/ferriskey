import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const createRoleSchema = z
  .object({
    name: z.string().min(1, { error: () => translate('role:validation.name_required') }),
    scope: z.enum(['realm', 'client']),
    clientId: z.string().optional(),
    description: z.string().optional(),
    permissions: z.array(z.string()),
  })
  .superRefine((value, ctx) => {
    if (value.scope === 'client' && (!value.clientId || value.clientId.trim().length === 0)) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: ['clientId'],
        message: translate('role:validation.client_id_required'),
      })
    }
  })

export type CreateRoleSchema = z.infer<typeof createRoleSchema>
