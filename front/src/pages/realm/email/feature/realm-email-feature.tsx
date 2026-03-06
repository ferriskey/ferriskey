import { useState } from 'react'
import { RealmEmail } from '../ui/realm-email'

export interface SmtpConfig {
  host: string
  port: string
  username: string
  password: string
  useTls: boolean
  fromEmail: string
  fromName: string
}

export interface EmailTemplatesConfig {
  verificationSubject: string
  verificationBody: string
  resetPasswordSubject: string
  resetPasswordBody: string
}

const defaultSmtp: SmtpConfig = {
  host: '',
  port: '587',
  username: '',
  password: '',
  useTls: true,
  fromEmail: '',
  fromName: 'FerrisKey',
}

const defaultTemplates: EmailTemplatesConfig = {
  verificationSubject: 'Verify your email address',
  verificationBody:
    'Hello {{username}},\n\nPlease verify your email address by clicking the link below:\n\n{{verificationLink}}\n\nThis link will expire in 24 hours.',
  resetPasswordSubject: 'Reset your password',
  resetPasswordBody:
    'Hello {{username}},\n\nYou requested a password reset. Click the link below to set a new password:\n\n{{resetLink}}\n\nThis link will expire in 1 hour. If you did not request this, please ignore this email.',
}

export default function RealmEmailFeature() {
  const [smtp, setSmtp] = useState<SmtpConfig>(defaultSmtp)
  const [templates, setTemplates] = useState<EmailTemplatesConfig>(defaultTemplates)

  const handleSmtpChange = (field: keyof SmtpConfig, value: string | boolean) => {
    setSmtp((prev) => ({ ...prev, [field]: value }))
  }

  const handleTemplateChange = (field: keyof EmailTemplatesConfig, value: string) => {
    setTemplates((prev) => ({ ...prev, [field]: value }))
  }

  const handleSave = () => {
    console.log('Saving email settings:', { smtp, templates })
  }

  const handleTestConnection = () => {
    alert('SMTP connection test triggered (mock)')
  }

  const handleSendTestEmail = () => {
    alert('Test email sent (mock)')
  }

  return (
    <RealmEmail
      smtp={smtp}
      templates={templates}
      onSmtpChange={handleSmtpChange}
      onTemplateChange={handleTemplateChange}
      onSave={handleSave}
      onTestConnection={handleTestConnection}
      onSendTestEmail={handleSendTestEmail}
    />
  )
}
