import { z } from 'zod'

export const createKerberosProviderSchema = z.object({
  name: z.string().min(1, 'Provider name is required'),
  type: z.literal('Kerberos'),
  enabled: z.boolean(),
  priority: z.enum(['Primary', 'Secondary', 'Development', 'Legacy']),
  kerberosRealm: z.string().min(1, 'Kerberos realm is required'),
  kdcServer: z.string().min(1, 'KDC server is required'),
  adminServer: z.string().optional(),
  allowPasswordAuth: z.boolean(),
})

export type CreateKerberosProviderSchema = z.infer<typeof createKerberosProviderSchema>
