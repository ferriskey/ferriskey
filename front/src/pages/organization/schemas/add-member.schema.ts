import { z } from 'zod'

export const addMemberSchema = z.object({
  userIds: z.array(z.string()).min(1, { message: 'At least one user must be selected' }),
})

export type AddMemberSchema = z.infer<typeof addMemberSchema>
