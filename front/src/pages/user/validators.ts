import { z } from "zod"

export const createUserValidator = z.object({
  username: z.string().min(1),
  firstname: z.string().min(1),
  lastname: z.string().min(1),
  email: z.string().email().min(1),
  email_verified: z.boolean().optional(),
})

export type CreateUserSchema = z.infer<typeof createUserValidator>