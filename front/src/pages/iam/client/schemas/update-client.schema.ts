import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const updateClientSchema = z.object({
  clientId: z.string().min(1, { error: () => translate('client:validation.client_id_required') }),
  name: z.string().min(1, { error: () => translate('client:validation.name_required') }),
  enabled: z.boolean().optional(),
  directAccessGrantsEnabled: z.boolean().optional(),
  oauthDeviceCodeGrantEnabled: z.boolean().optional(),
  tokenExchangeEnabled: z.boolean().optional(),
  requirePkce: z.boolean().optional(),
  accessTokenLifetime: z.number().nullable().optional(),
  refreshTokenLifetime: z.number().nullable().optional(),
  idTokenLifetime: z.number().nullable().optional(),
  temporaryTokenLifetime: z.number().nullable().optional(),
  backchannelLogoutUri: z
    .string()
    .refine((value) => value.trim() === '' || /^https?:\/\//i.test(value.trim()), {
      error: () => translate('client:validation.backchannel_logout_uri_invalid'),
    })
    .optional(),
  backchannelLogoutSessionRequired: z.boolean().optional(),
})

export type UpdateClientSchema = z.infer<typeof updateClientSchema>
