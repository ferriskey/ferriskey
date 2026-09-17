import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const createClientScopeSchema = z.object({
  name: z.string().min(1, { error: () => translate('client-scope:validation.scope_name_required') }),
  description: z.string().optional(),
  protocol: z.literal('openid-connect'),
  scopeType: z.enum(['optional', 'default']),
})

export type CreateClientScopeSchema = z.infer<typeof createClientScopeSchema>
