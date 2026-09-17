import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const updateClientScopeSchema = z.object({
  name: z.string().min(1, { error: () => translate('client-scope:validation.scope_name_required') }),
  description: z.string().optional(),
  scopeType: z.enum(['optional', 'default']),
})

export type UpdateClientScopeSchema = z.infer<typeof updateClientScopeSchema>
