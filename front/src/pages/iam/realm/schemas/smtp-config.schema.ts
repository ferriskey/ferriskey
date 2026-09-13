import { z } from 'zod'

export const smtpConfigSchema = z.object({
  host: z.string().min(1, 'Host is required'),
  port: z.number().min(1).max(65535),
  username: z.string().min(1, 'Username is required'),
  password: z.string().min(1, 'Password is required'),
  from_email: z.email('Must be a valid email'),
  from_name: z.string().min(1, 'From name is required'),
  encryption: z.enum(['tls', 'starttls', 'none']),
})

export type SmtpConfigSchema = z.infer<typeof smtpConfigSchema>
