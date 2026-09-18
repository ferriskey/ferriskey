import {
  smtpConfigSchema,
  type SmtpConfigSchema,
} from '@/pages/iam/realm/schemas/smtp-config.schema'
import { translate } from '@/lib/i18n'
import { EMAIL_TEMPLATE_NAMESPACE } from './email-types'

export type SmtpConfigDraft = SmtpConfigSchema

export const DEFAULT_SMTP_PORT = 587

export const SMTP_HOST_PLACEHOLDER = 'smtp.example.com'

export const SMTP_PASSWORD_PLACEHOLDER = '••••••••'

export const SMTP_DELETE_TOKEN = 'delete'

export const SMTP_FIELD = {
  host: 'host',
  port: 'port',
  username: 'username',
  password: 'password',
  fromEmail: 'from_email',
  fromName: 'from_name',
  encryption: 'encryption',
} as const satisfies Record<string, keyof SmtpConfigDraft>

export const EMPTY_SMTP_DRAFT: SmtpConfigDraft = {
  host: '',
  port: DEFAULT_SMTP_PORT,
  username: '',
  password: '',
  from_email: '',
  from_name: '',
  encryption: 'tls',
}

export const ENCRYPTIONS: {
  value: SmtpConfigDraft['encryption']
  labelKey: string
  hintKey: string
}[] = [
  {
    value: 'tls',
    labelKey: 'smtp.server.encryption.tls.label',
    hintKey: 'smtp.server.encryption.tls.hint',
  },
  {
    value: 'starttls',
    labelKey: 'smtp.server.encryption.starttls.label',
    hintKey: 'smtp.server.encryption.starttls.hint',
  },
  {
    value: 'none',
    labelKey: 'smtp.server.encryption.none.label',
    hintKey: 'smtp.server.encryption.none.hint',
  },
]

const FIELD_ERROR_KEYS: Record<keyof SmtpConfigDraft, string> = {
  host: 'smtp.validation.host',
  port: 'smtp.validation.port',
  username: 'smtp.validation.username',
  password: 'smtp.validation.password',
  from_email: 'smtp.validation.from_email',
  from_name: 'smtp.validation.from_name',
  encryption: 'smtp.validation.encryption',
}

export const smtpErrors = (draft: SmtpConfigDraft): Partial<Record<keyof SmtpConfigDraft, string>> => {
  const parsed = smtpConfigSchema.safeParse(draft)
  if (parsed.success) return {}

  const errors: Partial<Record<keyof SmtpConfigDraft, string>> = {}
  for (const issue of parsed.error.issues) {
    const key = issue.path[0] as keyof SmtpConfigDraft
    if (key && !errors[key]) {
      errors[key] = translate(`${EMAIL_TEMPLATE_NAMESPACE}:${FIELD_ERROR_KEYS[key]}`)
    }
  }
  return errors
}
