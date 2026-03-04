import { Plug } from 'lucide-react'
import BlockContent from '@/components/ui/block-content'
import { InputText } from '@/components/ui/input-text'
import { Switch } from '@/components/ui/switch'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import type { SmtpConfig } from '../feature/realm-email-feature'

export interface SmtpConnectionFormProps {
  config: SmtpConfig
  onChange: (field: keyof SmtpConfig, value: string | boolean) => void
  onTestConnection: () => void
}

export function SmtpConnectionForm({ config, onChange, onTestConnection }: SmtpConnectionFormProps) {
  return (
    <div className='flex flex-col'>
      <BlockContent
        title='SMTP Server'
        headRight={
          <Button type='button' variant='outline' size='sm' onClick={onTestConnection}>
            <Plug className='h-3.5 w-3.5 mr-1.5' />
            Test Connection
          </Button>
        }
        headHeight='h-11'
      >
        <div className='flex flex-col gap-5'>
          <div className='grid grid-cols-2 gap-4'>
            <InputText
              name='host'
              label='Host'
              value={config.host}
              onChange={(val) => onChange('host', String(val))}
            />
            <InputText
              name='port'
              label='Port'
              value={config.port}
              onChange={(val) => onChange('port', String(val))}
            />
          </div>

          <div className='flex items-center justify-between rounded-md border p-3 bg-background'>
            <div className='space-y-0.5'>
              <Label className='text-sm font-medium'>Use TLS</Label>
              <p className='text-xs text-muted-foreground'>
                Enable TLS encryption for the SMTP connection.
              </p>
            </div>
            <Switch
              checked={config.useTls}
              onCheckedChange={(checked) => onChange('useTls', checked)}
            />
          </div>
        </div>
      </BlockContent>

      <BlockContent title='Authentication'>
        <div className='grid grid-cols-2 gap-4'>
          <InputText
            name='username'
            label='Username'
            value={config.username}
            onChange={(val) => onChange('username', String(val))}
          />
          <InputText
            name='password'
            label='Password'
            type='password'
            value={config.password}
            onChange={(val) => onChange('password', String(val))}
          />
        </div>
      </BlockContent>

      <BlockContent title='Sender Identity'>
        <div className='grid grid-cols-2 gap-4'>
          <InputText
            name='fromEmail'
            label='From Email'
            value={config.fromEmail}
            onChange={(val) => onChange('fromEmail', String(val))}
          />
          <InputText
            name='fromName'
            label='From Name'
            value={config.fromName}
            onChange={(val) => onChange('fromName', String(val))}
          />
        </div>
      </BlockContent>
    </div>
  )
}
