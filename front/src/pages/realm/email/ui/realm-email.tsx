import { Button } from '@/components/ui/button'
import { SmtpConnectionForm } from './smtp-connection-form'
import { EmailTemplatesForm } from './email-templates-form'
import type { SmtpConfig, EmailTemplatesConfig } from '../feature/realm-email-feature'

export interface RealmEmailProps {
  smtp: SmtpConfig
  templates: EmailTemplatesConfig
  onSmtpChange: (field: keyof SmtpConfig, value: string | boolean) => void
  onTemplateChange: (field: keyof EmailTemplatesConfig, value: string) => void
  onSave: () => void
  onTestConnection: () => void
  onSendTestEmail: () => void
}

export function RealmEmail({
  smtp,
  templates,
  onSmtpChange,
  onTemplateChange,
  onSave,
  onTestConnection,
  onSendTestEmail,
}: RealmEmailProps) {
  return (
    <div className='flex flex-col gap-2'>
      <div className='mb-2'>
        <p className='text-xs text-muted-foreground mb-0.5'>Realm configuration</p>
        <h2 className='text-base font-semibold'>Email Settings</h2>
      </div>

      <SmtpConnectionForm
        config={smtp}
        onChange={onSmtpChange}
        onTestConnection={onTestConnection}
      />

      <EmailTemplatesForm
        templates={templates}
        onChange={onTemplateChange}
        onSendTestEmail={onSendTestEmail}
      />

      <div className='flex justify-end pt-2 pb-4'>
        <Button type='button' onClick={onSave}>
          Save Settings
        </Button>
      </div>
    </div>
  )
}
