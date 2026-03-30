import { UseFormReturn } from 'react-hook-form'
import { SmtpConfigSchema } from '../feature/page-realm-settings-smtp-feature'
import { Form, FormField } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Label } from '@/components/ui/label'
import { DangerZone } from '@/components/danger-zone'
import { Button } from '@/components/ui/button'
import { Schemas } from '@/api/api.client'

interface PageRealmSettingsSmtpProps {
  form: UseFormReturn<SmtpConfigSchema>
  config?: Schemas.SmtpConfig
  handleSubmit: (values: SmtpConfigSchema) => void
  handleDelete: () => void
}

function SmtpConfigDisplay({ config, onDelete }: { config: Schemas.SmtpConfig; onDelete: () => void }) {
  const fields = [
    { label: 'Host', description: 'SMTP server hostname.', value: config.host },
    { label: 'Port', description: 'SMTP server port.', value: String(config.port) },
    { label: 'Encryption', description: 'Connection encryption method.', value: config.encryption.toUpperCase() },
    { label: 'Username', description: 'SMTP authentication username.', value: config.username },
    { label: 'From Email', description: 'Sender email address for outgoing emails.', value: config.from_email },
    { label: 'From Name', description: 'Display name for the sender.', value: config.from_name },
  ]

  return (
    <div className='flex flex-col gap-8'>
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Email delivery configuration</p>
          <h2 className='text-base font-semibold'>SMTP Settings</h2>
        </div>

        {fields.map((field) => (
          <div key={field.label} className='flex items-start justify-between py-4 border-t'>
            <div className='w-1/3'>
              <p className='text-sm font-medium'>{field.label}</p>
              <p className='text-sm text-muted-foreground mt-0.5'>{field.description}</p>
            </div>
            <div className='w-1/2'>
              <p className='text-sm'>{field.value}</p>
            </div>
          </div>
        ))}
      </div>

      <DangerZone
        label='Delete SMTP configuration'
        description='Remove the SMTP configuration for this realm. Email features (password reset, magic links) will stop working.'
        buttonLabel='Delete SMTP config'
        confirmTitle='Delete SMTP configuration'
        confirmDescription='This will permanently remove the SMTP configuration for this realm. Email delivery will be disabled.'
        confirmText='delete'
        onConfirm={onDelete}
      />
    </div>
  )
}

function SmtpConfigForm({
  form,
  handleSubmit,
}: {
  form: UseFormReturn<SmtpConfigSchema>
  handleSubmit: (values: SmtpConfigSchema) => void
}) {
  return (
    <Form {...form}>
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Email delivery configuration</p>
          <h2 className='text-base font-semibold'>SMTP Settings</h2>
        </div>

        <FormField
          control={form.control}
          name='host'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Host</p>
                <p className='text-sm text-muted-foreground mt-0.5'>SMTP server hostname.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Host' {...field} error={form.formState.errors.host?.message} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='port'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Port</p>
                <p className='text-sm text-muted-foreground mt-0.5'>SMTP server port (e.g. 587, 465, 25).</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Port' type='number' {...field} value={String(field.value)} error={form.formState.errors.port?.message} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='encryption'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Encryption</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Connection encryption method.</p>
              </div>
              <div className='w-1/2'>
                <Label className='text-sm text-muted-foreground mb-1.5 block'>Encryption</Label>
                <Select onValueChange={field.onChange} value={field.value}>
                  <SelectTrigger className='w-48'>
                    <SelectValue>{field.value.toUpperCase()}</SelectValue>
                  </SelectTrigger>
                  <SelectContent position='popper'>
                    <SelectItem value='tls'>TLS</SelectItem>
                    <SelectItem value='starttls'>STARTTLS</SelectItem>
                    <SelectItem value='none'>None</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='username'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Username</p>
                <p className='text-sm text-muted-foreground mt-0.5'>SMTP authentication username.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Username' {...field} error={form.formState.errors.username?.message} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='password'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Password</p>
                <p className='text-sm text-muted-foreground mt-0.5'>SMTP authentication password.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Password' type='password' {...field} error={form.formState.errors.password?.message} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='from_email'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>From Email</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Sender email address for outgoing emails.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='From Email' {...field} error={form.formState.errors.from_email?.message} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='from_name'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>From Name</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Display name for the sender.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='From Name' {...field} error={form.formState.errors.from_name?.message} />
              </div>
            </div>
          )}
        />

        <div className='flex justify-end pt-4 border-t'>
          <Button type='button' onClick={() => form.handleSubmit(handleSubmit)()}>
            Save
          </Button>
        </div>
      </div>
    </Form>
  )
}

export default function PageRealmSettingsSmtp({
  form,
  config,
  handleSubmit,
  handleDelete,
}: PageRealmSettingsSmtpProps) {
  return (
    <div className='flex flex-col gap-10'>
      {config ? (
        <SmtpConfigDisplay config={config} onDelete={handleDelete} />
      ) : (
        <SmtpConfigForm form={form} handleSubmit={handleSubmit} />
      )}
    </div>
  )
}
