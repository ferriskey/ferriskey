import * as React from 'react'
import { Send } from 'lucide-react'
import BlockContent from '@/components/ui/block-content'
import { InputText } from '@/components/ui/input-text'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { cn } from '@/lib/utils'
import type { EmailTemplatesConfig } from '../feature/realm-email-feature'

function Textarea({ className, ...props }: React.ComponentProps<'textarea'>) {
  return (
    <textarea
      className={cn(
        'border-input placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50',
        'flex min-h-[120px] w-full rounded-md border bg-background px-3 py-2 text-sm',
        'shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px]',
        'disabled:cursor-not-allowed disabled:opacity-50 resize-y font-mono',
        className
      )}
      {...props}
    />
  )
}

export interface EmailTemplatesFormProps {
  templates: EmailTemplatesConfig
  onChange: (field: keyof EmailTemplatesConfig, value: string) => void
  onSendTestEmail: () => void
}

export function EmailTemplatesForm({ templates, onChange, onSendTestEmail }: EmailTemplatesFormProps) {
  return (
    <div className='flex flex-col'>
      <BlockContent
        title='Email Verification'
        headRight={
          <Button type='button' variant='outline' size='sm' onClick={onSendTestEmail}>
            <Send className='h-3.5 w-3.5 mr-1.5' />
            Send Test Email
          </Button>
        }
        headHeight='h-11'
      >
        <div className='flex flex-col gap-5'>
          <InputText
            name='verificationSubject'
            label='Subject'
            value={templates.verificationSubject}
            onChange={(val) => onChange('verificationSubject', String(val))}
          />
          <div className='flex flex-col gap-1.5'>
            <Label className='text-sm text-muted-foreground'>Body</Label>
            <Textarea
              rows={5}
              placeholder='Hello {{username}}, please verify your email by clicking the link below...'
              value={templates.verificationBody}
              onChange={(e) => onChange('verificationBody', e.target.value)}
            />
            <p className='text-xs text-muted-foreground'>
              Available variables: <code className='bg-muted px-1 rounded'>{'{{username}}'}</code>{' '}
              <code className='bg-muted px-1 rounded'>{'{{verificationLink}}'}</code>
            </p>
          </div>
        </div>
      </BlockContent>

      <BlockContent title='Password Reset'>
        <div className='flex flex-col gap-5'>
          <InputText
            name='resetPasswordSubject'
            label='Subject'
            value={templates.resetPasswordSubject}
            onChange={(val) => onChange('resetPasswordSubject', String(val))}
          />
          <div className='flex flex-col gap-1.5'>
            <Label className='text-sm text-muted-foreground'>Body</Label>
            <Textarea
              rows={5}
              placeholder='Hello {{username}}, click the link below to reset your password...'
              value={templates.resetPasswordBody}
              onChange={(e) => onChange('resetPasswordBody', e.target.value)}
            />
            <p className='text-xs text-muted-foreground'>
              Available variables: <code className='bg-muted px-1 rounded'>{'{{username}}'}</code>{' '}
              <code className='bg-muted px-1 rounded'>{'{{resetLink}}'}</code>
            </p>
          </div>
        </div>
      </BlockContent>
    </div>
  )
}
