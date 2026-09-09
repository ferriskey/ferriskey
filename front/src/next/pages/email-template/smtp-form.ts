import {
  smtpConfigSchema,
  type SmtpConfigSchema,
} from '@/pages/realm/schemas/smtp-config.schema'

export type SmtpConfigDraft = SmtpConfigSchema

export const EMPTY_SMTP_DRAFT: SmtpConfigDraft = {
  host: '',
  port: 587,
  username: '',
  password: '',
  from_email: '',
  from_name: '',
  encryption: 'tls',
}

export const ENCRYPTIONS: { value: SmtpConfigDraft['encryption']; label: string; hint: string }[] = [
  {
    value: 'tls',
    label: 'TLS',
    hint: 'Encrypted from the first byte, usually on port 465.',
  },
  {
    value: 'starttls',
    label: 'STARTTLS',
    hint: 'Plain connection upgraded to encryption, usually on port 587.',
  },
  {
    value: 'none',
    label: 'None',
    hint: 'In the clear. Only for a local relay.',
  },
]

export const smtpErrors = (draft: SmtpConfigDraft): Partial<Record<keyof SmtpConfigDraft, string>> => {
  const parsed = smtpConfigSchema.safeParse(draft)
  if (parsed.success) return {}

  const errors: Partial<Record<keyof SmtpConfigDraft, string>> = {}
  for (const issue of parsed.error.issues) {
    const key = issue.path[0] as keyof SmtpConfigDraft
    if (key && !errors[key]) errors[key] = issue.message
  }
  return errors
}
