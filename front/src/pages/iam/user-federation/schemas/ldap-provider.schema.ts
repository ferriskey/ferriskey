import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const createLdapProviderSchema = z.object({
  name: z
    .string()
    .min(1, { error: () => translate('user-federation:validation.name_required') }),
  type: z.literal('LDAP'),
  enabled: z.boolean(),
  priority: z.enum(['Primary', 'Secondary', 'Development', 'Legacy']),
  connectionUrl: z
    .string()
    .min(1, { error: () => translate('user-federation:validation.connection_url_required') }),
  baseDn: z
    .string()
    .min(1, { error: () => translate('user-federation:validation.base_dn_required') }),
  bindDn: z.string().optional(),
  bindPassword: z.string().optional(),
  userSearchFilter: z.string(),
  syncInterval: z.number().min(60),
  useTls: z.boolean(),
})

export type CreateLdapProviderSchema = z.infer<typeof createLdapProviderSchema>
