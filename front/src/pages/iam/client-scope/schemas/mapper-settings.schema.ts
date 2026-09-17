import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const mapperSettingsSchema = z.object({
  name: z
    .string()
    .min(1, { error: () => translate('client-scope:validation.mapper_name_required') }),
  config_json: z
    .string()
    .optional()
    .refine(
      (val) => {
        if (!val || val.trim() === '') return true
        try {
          JSON.parse(val)
          return true
        } catch {
          return false
        }
      },
      { error: () => translate('client-scope:validation.mapper_config_invalid_json') }
    ),
})

export type MapperSettingsSchema = z.infer<typeof mapperSettingsSchema>
