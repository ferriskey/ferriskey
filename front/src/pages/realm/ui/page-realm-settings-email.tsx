import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
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
  templates: EmailTemplate[]
  templatesLoading: boolean
  onEditTemplate: (id: string) => void
  onCreateTemplate: () => void
  onDeleteTemplate: (id: string) => void
  realmSettings?: RealmSettings | null
  onAssignTemplate: (field: string, templateId: string | null) => void
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
            <div className='w-1/2 flex justify-end'>
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
      <EmailActionsSection
        templates={templates}
        realmSettings={realmSettings}
        onAssignTemplate={onAssignTemplate}
      />

      <EmailTemplatesSection
        templates={templates}
        isLoading={templatesLoading}
        onEdit={onEditTemplate}
        onCreate={onCreateTemplate}
        onDelete={onDeleteTemplate}
      />
    </div>
  )
}
