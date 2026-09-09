import { z } from 'zod'

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
  name: z.string().min(1, 'Name is required'),
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
      { message: 'Config must be valid JSON' }
    ),
})

export type MapperTemplateFormSchema = z.infer<typeof mapperTemplateFormSchema>
