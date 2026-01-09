import { z } from 'zod'

export const createLdapProviderSchema = z.object({
  name: z.string().min(1, 'Provider name is required'),
  type: z.literal('LDAP'),
  enabled: z.boolean(),
  priority: z.enum(['Primary', 'Secondary', 'Development', 'Legacy']),
  connectionUrl: z.string().min(1, 'Connection URL is required'),
  baseDn: z.string().min(1, 'Base DN is required'),
  bindDn: z.string().optional(),
  bindPassword: z.string().optional(),
  userSearchFilter: z.string(),
  syncInterval: z.coerce.number().min(60),
  useTls: z.boolean(),
})

export type CreateLdapProviderSchema = z.infer<typeof createLdapProviderSchema>
