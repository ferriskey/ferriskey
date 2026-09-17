import { KeyRound, Link2, MailCheck, type LucideIcon } from 'lucide-react'
import { Schemas } from '@/api/api.client'

import EmailType = Schemas.EmailType
import { formatDate } from '@/utils/format-date'
import { preloadNamespaces, translate } from '@/lib/i18n'

export const EMAIL_TEMPLATE_NAMESPACE = 'email-template'

void preloadNamespaces(EMAIL_TEMPLATE_NAMESPACE).catch(() => undefined)

export const DEFAULT_EMAIL_TYPE: EmailType = 'reset_password'

export const ALL_EMAIL_TYPES = 'all'

export const EXPORT_JSON = 'json'

export const EXPORT_MJML = 'mjml'

const UNKNOWN_DATE = '—'

export interface EmailTypeSpec {
  key: EmailType
  labelKey: string
  shortKey: string
  icon: LucideIcon
  tone: 'amber' | 'violet' | 'success'
  assignmentField:
    | 'reset_password_template_id'
    | 'magic_link_template_id'
    | 'email_verification_template_id'
  triggerKey: string
}

export const EMAIL_TYPES: EmailTypeSpec[] = [
  {
    key: 'reset_password',
    labelKey: 'type.reset_password.label',
    shortKey: 'type.reset_password.short',
    icon: KeyRound,
    tone: 'amber',
    assignmentField: 'reset_password_template_id',
    triggerKey: 'type.reset_password.trigger',
  },
  {
    key: 'magic_link',
    labelKey: 'type.magic_link.label',
    shortKey: 'type.magic_link.short',
    icon: Link2,
    tone: 'violet',
    assignmentField: 'magic_link_template_id',
    triggerKey: 'type.magic_link.trigger',
  },
  {
    key: 'email_verification',
    labelKey: 'type.email_verification.label',
    shortKey: 'type.email_verification.short',
    icon: MailCheck,
    tone: 'success',
    assignmentField: 'email_verification_template_id',
    triggerKey: 'type.email_verification.trigger',
  },
]

export const emailTypeSpec = (type: string): EmailTypeSpec =>
  EMAIL_TYPES.find((t) => t.key === type) ?? EMAIL_TYPES[0]

export const variableToken = (name: string) => `{{${name}}}`

export const formatRelative = (iso: string) => {
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return UNKNOWN_DATE
  const seconds = Math.round((Date.now() - date.getTime()) / 1000)
  if (seconds < 60) return translate(`${EMAIL_TEMPLATE_NAMESPACE}:relative.just_now`)
  const minutes = Math.round(seconds / 60)
  if (minutes < 60) {
    return translate(`${EMAIL_TEMPLATE_NAMESPACE}:relative.minutes`, { total: minutes })
  }
  const hours = Math.round(minutes / 60)
  if (hours < 24) {
    return translate(`${EMAIL_TEMPLATE_NAMESPACE}:relative.hours`, { total: hours })
  }
  const days = Math.round(hours / 24)
  if (days < 30) {
    return translate(`${EMAIL_TEMPLATE_NAMESPACE}:relative.days`, { total: days })
  }
  return formatDate(date.toISOString())
}

export const citedVariables = (mjml: string): Set<string> => {
  const cited = new Set<string>()
  for (const match of mjml.matchAll(/\{\{([^{}]*)\}\}/g)) {
    cited.add(match[1])
  }
  return cited
}
