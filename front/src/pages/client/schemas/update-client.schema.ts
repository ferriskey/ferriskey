import { z } from "zod";

export const updateClientSchema = z.object({
  clientId: z.string().min(1, { message: "The client ID is required" }),
  name: z.string().min(1, { message: "The name is required" }),
  enabled: z.boolean().optional(),
})

export type UpdateClientSchema = z.infer<typeof updateClientSchema>