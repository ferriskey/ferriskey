import { z } from 'zod'

export const mapperSettingsSchema = z.object({
  name: z.string().min(1, 'Name is required'),
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

export type MapperSettingsSchema = z.infer<typeof mapperSettingsSchema>
