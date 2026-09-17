import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const providerTypeSchema = z.enum(['oidc', 'oauth2', 'saml', 'ldap'])

export const createProviderSchema = z.object({
  alias: z
    .string()
    .min(1, { error: () => translate('identity-provider:validation.alias_required') })
    .regex(/^[a-z0-9-]+$/, {
      error: () => translate('identity-provider:validation.alias_format'),
    }),
  displayName: z
    .string()
    .min(1, { error: () => translate('identity-provider:validation.display_name_required') }),
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
