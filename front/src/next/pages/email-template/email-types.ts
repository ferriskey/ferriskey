import { KeyRound, Link2, MailCheck, type LucideIcon } from 'lucide-react'
import { Schemas } from '@/api/api.client'

import EmailType = Schemas.EmailType

export interface EmailTypeSpec {
  key: EmailType
  label: string
  short: string
  icon: LucideIcon
  tone: 'amber' | 'violet' | 'success'
  assignmentField:
    | 'reset_password_template_id'
    | 'magic_link_template_id'
    | 'email_verification_template_id'
  trigger: string
}

export const EMAIL_TYPES: EmailTypeSpec[] = [
  {
    key: 'reset_password',
    label: 'Reset Password',
    short: 'Password',
    icon: KeyRound,
    tone: 'amber',
    assignmentField: 'reset_password_template_id',
    trigger: 'Sent when a user asks to reset their password.',
  },
  {
    key: 'magic_link',
    label: 'Magic Link',
    short: 'Magic Link',
    icon: Link2,
    tone: 'violet',
    assignmentField: 'magic_link_template_id',
    trigger: 'Sent when a user asks for a sign-in link.',
  },
  {
    key: 'email_verification',
    label: 'Email Verification',
    short: 'Verification',
    icon: MailCheck,
    tone: 'success',
    assignmentField: 'email_verification_template_id',
    trigger: 'Sent when an account is created, or on a verification request.',
  },
]

export const emailTypeSpec = (type: string): EmailTypeSpec =>
  EMAIL_TYPES.find((t) => t.key === type) ?? EMAIL_TYPES[0]

export const formatRelative = (iso: string) => {
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return '—'
  const seconds = Math.round((Date.now() - date.getTime()) / 1000)
  if (seconds < 60) return 'just now'
  const minutes = Math.round(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.round(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.round(hours / 24)
  if (days < 30) return `${days}d ago`
  return date.toLocaleDateString()
}

export const citedVariables = (mjml: string): Set<string> => {
  const cited = new Set<string>()
  for (const match of mjml.matchAll(/\{\{([^{}]*)\}\}/g)) {
    cited.add(match[1])
  }
  return cited
}
