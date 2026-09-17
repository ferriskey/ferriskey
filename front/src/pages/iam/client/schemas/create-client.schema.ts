import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const createClientSchema = z.object({
  clientId: z.string().min(1, { error: () => translate('client:validation.client_id_required') }),
  name: z.string().min(1, { error: () => translate('client:validation.name_required') }),
  enabled: z.boolean().optional(),
  clientAuthentication: z.boolean().optional(),
  protocol: z.enum(['openid-connect', 'saml']),
})

export type CreateClientSchema = z.infer<typeof createClientSchema>
