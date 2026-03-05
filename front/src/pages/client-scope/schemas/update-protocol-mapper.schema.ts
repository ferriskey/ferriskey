import { z } from 'zod'

export const updateProtocolMapperSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  mapper_type: z.string().min(1, 'Mapper type is required'),
  config: z
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

export type UpdateProtocolMapperSchema = z.infer<typeof updateProtocolMapperSchema>
