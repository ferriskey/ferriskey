import { z } from 'zod'
import { translate } from '@/lib/i18n'
import { AUTH_NAMESPACE } from '../constants'

export const magicLinkSchema = z.object({
  email: z
    .string()
    .min(1, { error: () => translate(`${AUTH_NAMESPACE}:validation.email_required`) })
    .email({ error: () => translate(`${AUTH_NAMESPACE}:validation.email_invalid_hint`) }),
})

export type MagicLinkSchema = z.infer<typeof magicLinkSchema>
