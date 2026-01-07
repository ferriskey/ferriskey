import { z } from 'zod'

export const createProviderSchema = z.object({
  name: z.string().min(1, 'Provider name is required'),
  type: z.enum(['LDAP', 'ActiveDirectory', 'Kerberos', 'Custom']),
  priority: z.enum(['Primary', 'Secondary', 'Development', 'Legacy', 'Custom']),
  enabled: z.boolean().default(true),
  connectionUrl: z.string().min(1, 'Connection URL is required'),
  baseDn: z.string().min(1, 'Base DN is required'),
  bindDn: z.string().optional(),
  bindPassword: z.string().optional(),
  userSearchBase: z.string().optional(),
  userSearchFilter: z.string().default('(objectClass=person)'),
  syncInterval: z.number().min(60, 'Sync interval must be at least 60 seconds').default(3600),
})

export type CreateProviderSchema = z.infer<typeof createProviderSchema>
