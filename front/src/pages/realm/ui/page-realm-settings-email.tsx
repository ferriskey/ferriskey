import { UseFormReturn } from 'react-hook-form'
import { SmtpConfigSchema } from '../feature/page-realm-settings-email-feature'
import { Form, FormField } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Label } from '@/components/ui/label'
import { DangerZone } from '@/components/danger-zone'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Schemas } from '@/api/api.client'
import { Mail, Pencil, Plus, Trash2 } from 'lucide-react'

interface EmailTemplate {
  id: string
  name: string
  email_type: string
}

interface RealmSettings {
  reset_password_template_id?: string | null
  magic_link_template_id?: string | null
  email_verification_template_id?: string | null
}

export interface PageRealmSettingsEmailProps {
  form: UseFormReturn<SmtpConfigSchema>
  config?: Schemas.SmtpConfig
  handleSubmit: (values: SmtpConfigSchema) => void
  handleDelete: () => void
  templates: EmailTemplate[]
  templatesLoading: boolean
  onEditTemplate: (id: string) => void
  onCreateTemplate: () => void
  onDeleteTemplate: (id: string) => void
  realmSettings?: RealmSettings | null
  onAssignTemplate: (field: string, templateId: string | null) => void
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

const EMAIL_TYPE_LABELS: Record<string, string> = {
  reset_password: 'Reset Password',
  magic_link: 'Magic Link',
  email_verification: 'Email Verification',
}

function EmailTemplatesSection({
  templates,
  isLoading,
  onEdit,
  onCreate,
  onDelete,
}: {
  templates: EmailTemplate[]
  isLoading: boolean
  onEdit: (id: string) => void
  onCreate: () => void
  onDelete: (id: string) => void
}) {
  return (
    <div className='flex flex-col gap-4'>
      <div className='flex items-center justify-between'>
        <div>
          <p className='text-xs text-muted-foreground mb-0.5'>Email appearance</p>
          <h2 className='text-base font-semibold'>Email Templates</h2>
        </div>
        <Button variant='outline' size='sm' onClick={onCreate}>
          <Plus size={14} />
          New Template
        </Button>
      </div>

      {isLoading ? (
        <p className='text-sm text-muted-foreground py-4'>Loading templates...</p>
      ) : templates.length === 0 ? (
        <div className='flex flex-col items-center gap-2 rounded-lg border border-dashed border-border py-8'>
          <Mail size={28} className='text-muted-foreground' />
          <p className='text-sm text-muted-foreground'>No email templates configured.</p>
          <Button variant='outline' size='sm' onClick={onCreate}>
            <Plus size={14} />
            Create Template
          </Button>
        </div>
      ) : (
        <div className='flex flex-col gap-2'>
          {templates.map((template) => (
            <div
              key={template.id}
              className='flex items-center justify-between rounded-lg border border-border px-4 py-3'
            >
              <div className='flex items-center gap-3'>
                <span className='text-sm font-medium'>{template.name}</span>
                <Badge variant='outline'>
                  {EMAIL_TYPE_LABELS[template.email_type] ?? template.email_type}
                </Badge>
              </div>
              <div className='flex items-center gap-1'>
                <Button variant='ghost' size='icon' title='Edit' onClick={() => onEdit(template.id)}>
                  <Pencil size={14} />
                </Button>
                <Button
                  variant='ghost'
                  size='icon'
                  title='Delete'
                  className='text-destructive hover:text-destructive'
                  onClick={() => onDelete(template.id)}
                >
                  <Trash2 size={14} />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

const EMAIL_ACTION_CONFIGS = [
  { field: 'reset_password_template_id', label: 'Reset Password', emailType: 'reset_password', description: 'Template used when a user requests a password reset.' },
  { field: 'magic_link_template_id', label: 'Magic Link', emailType: 'magic_link', description: 'Template used for passwordless magic link authentication.' },
  { field: 'email_verification_template_id', label: 'Email Verification', emailType: 'email_verification', description: 'Template used to verify a user\'s email address.' },
] as const

function EmailActionsSection({
  templates,
  realmSettings,
  onAssignTemplate,
}: {
  templates: EmailTemplate[]
  realmSettings?: RealmSettings | null
  onAssignTemplate: (field: string, templateId: string | null) => void
}) {
  return (
    <div className='flex flex-col gap-1'>
      <div className='mb-4'>
        <p className='text-xs text-muted-foreground mb-0.5'>Email routing</p>
        <h2 className='text-base font-semibold'>Email Actions Configuration</h2>
      </div>

      {EMAIL_ACTION_CONFIGS.map((action) => {
        const filtered = templates.filter((t) => t.email_type === action.emailType)
        const currentValue = realmSettings?.[action.field] ?? undefined

        return (
          <div key={action.field} className='flex items-start justify-between py-4 border-t'>
            <div className='w-1/3'>
              <p className='text-sm font-medium'>{action.label}</p>
              <p className='text-sm text-muted-foreground mt-0.5'>{action.description}</p>
            </div>
            <div className='w-1/2'>
              <Select
                value={currentValue ?? '__none__'}
                onValueChange={(value) =>
                  onAssignTemplate(action.field, value === '__none__' ? null : value)
                }
              >
                <SelectTrigger className='w-64'>
                  <SelectValue placeholder='Not configured' />
                </SelectTrigger>
                <SelectContent position='popper'>
                  <SelectItem value='__none__'>Not configured</SelectItem>
                  {filtered.map((t) => (
                    <SelectItem key={t.id} value={t.id}>
                      {t.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        )
      })}
    </div>
  )
}

export default function PageRealmSettingsEmail({
  form,
  config,
  handleSubmit,
  handleDelete,
  templates,
  templatesLoading,
  onEditTemplate,
  onCreateTemplate,
  onDeleteTemplate,
  realmSettings,
  onAssignTemplate,
}: PageRealmSettingsEmailProps) {
  return (
    <div className='flex flex-col gap-10'>
      {config ? (
        <SmtpConfigDisplay config={config} onDelete={handleDelete} />
      ) : (
        <SmtpConfigForm form={form} handleSubmit={handleSubmit} />
      )}

      <EmailTemplatesSection
        templates={templates}
        isLoading={templatesLoading}
        onEdit={onEditTemplate}
        onCreate={onCreateTemplate}
        onDelete={onDeleteTemplate}
      />

      <EmailActionsSection
        templates={templates}
        realmSettings={realmSettings}
        onAssignTemplate={onAssignTemplate}
      />
    </div>
  )
}
