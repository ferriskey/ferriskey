import { z } from 'zod'
import { translate } from '@/lib/i18n'

/**
 * Form schema for the template picker modal.
 *
 * - `name`        — mapper name, always required
 * - `mapper_type` — only used when isCustom = true
 * - `config_json` — raw JSON textarea, only used when isCustom = true
 *
 * Dynamic config fields (from ConfigFieldDef[]) are maintained as a separate
 * `Record<string, string>` state in the feature component to avoid react-hook-form
 * interpreting dotted keys (e.g. "token.claim.name") as nested object paths.
 */
export const mapperTemplateFormSchema = z.object({
  name: z
    .string()
    .min(1, { error: () => translate('client-scope:validation.mapper_name_required') }),
  mapper_type: z.string().optional(),
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

export type MapperTemplateFormSchema = z.infer<typeof mapperTemplateFormSchema>
