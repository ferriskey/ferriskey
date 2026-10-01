import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const tokenExchangePolicySchema = z.object({
  targetAudience: z
    .string()
    .min(1, { error: () => translate('client:validation.target_audience_required') }),
  allowedScopes: z.array(z.string()),
  allowImpersonation: z.boolean(),
  allowDelegation: z.boolean(),
})

export type TokenExchangePolicySchema = z.infer<typeof tokenExchangePolicySchema>
