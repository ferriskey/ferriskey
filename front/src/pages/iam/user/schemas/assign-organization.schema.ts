import { z } from 'zod'

export const assignOrganizationSchema = z.object({
  organizationIds: z.array(z.string()).min(1, { message: 'At least one organization must be selected' }),
})

export type AssignOrganizationSchema = z.infer<typeof assignOrganizationSchema>
