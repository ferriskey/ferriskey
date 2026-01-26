import { z } from 'zod'

export const providerTypeSchema = z.enum(['oidc', 'oauth2', 'saml', 'ldap'])

export const createProviderSchema = z.object({
  alias: z
    .string()
    .min(1, { message: 'Alias is required' })
    .regex(/^[a-z0-9-]+$/, {
      message: 'Alias must contain only lowercase letters, numbers, and hyphens',
    }),
  displayName: z.string().min(1, { message: 'Display name is required' }),
  providerType: providerTypeSchema,
  enabled: z.boolean().default(true),
  // OIDC/OAuth2 specific fields
  clientId: z.string().optional(),
  clientSecret: z.string().optional(),
  authorizationUrl: z.string().url().optional().or(z.literal('')),
  tokenUrl: z.string().url().optional().or(z.literal('')),
  userinfoUrl: z.string().url().optional().or(z.literal('')),
  // SAML specific fields
  entityId: z.string().optional(),
  ssoUrl: z.string().url().optional().or(z.literal('')),
  // LDAP specific fields
  ldapUrl: z.string().optional(),
  bindDn: z.string().optional(),
  bindCredential: z.string().optional(),
})

export type CreateProviderSchema = z.infer<typeof createProviderSchema>
