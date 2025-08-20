import { SigningAlgorithm } from '@/api/core.interface'
import { z } from 'zod'

export const updateRealmValidator = z.object({
  name: z.string().min(1),
  default_signing_algorithm: z.nativeEnum(SigningAlgorithm),
})

export type UpdateRealmSchema = z.infer<typeof updateRealmValidator>
