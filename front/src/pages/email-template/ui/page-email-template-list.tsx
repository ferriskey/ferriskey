import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Mail, Pencil, Power, Plus, Trash2 } from 'lucide-react'

interface EmailTemplate {
  id: string
  name: string
  email_type: string
  is_active: boolean
  created_at: string
  updated_at: string
}

interface Props {
  templates: EmailTemplate[]
  isLoading: boolean
  onEdit: (id: string) => void
  onDelete: (id: string) => void
  onActivate: (id: string) => void
  onCreate: () => void
}

const EMAIL_TYPE_LABELS: Record<string, string> = {
  reset_password: 'Reset Password',
  magic_link: 'Magic Link',
  email_verification: 'Email Verification',
}

export default function PageEmailTemplateList({
  templates,
  isLoading,
  onEdit,
  onDelete,
  onActivate,
  onCreate,
}: Props) {
  return (
    <div className='flex flex-col gap-4 p-6'>
      <div className='flex items-center justify-between'>
        <div>
          <h1 className='text-2xl font-semibold'>Email Templates</h1>
          <p className='text-sm text-muted-foreground'>
            Manage email templates for your realm
          </p>
        </div>
        <Button onClick={onCreate}>
          <Plus size={16} />
          New Template
        </Button>
      </div>

      {isLoading ? (
        <div className='flex items-center justify-center py-12 text-sm text-muted-foreground'>
          Loading templates...
        </div>
      ) : templates.length === 0 ? (
        <Card>
          <CardContent className='flex flex-col items-center justify-center gap-3 py-12'>
            <Mail size={40} className='text-muted-foreground' />
            <p className='text-sm text-muted-foreground'>
              No email templates yet. Create one to get started.
            </p>
            <Button variant='outline' onClick={onCreate}>
              <Plus size={16} />
              Create Template
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className='grid gap-3'>
          {templates.map((template) => (
            <Card key={template.id}>
              <CardHeader className='flex flex-row items-center justify-between pb-2'>
                <div className='flex items-center gap-3'>
                  <CardTitle className='text-base'>{template.name}</CardTitle>
                  <Badge variant='outline'>
                    {EMAIL_TYPE_LABELS[template.email_type] ?? template.email_type}
                  </Badge>
                  {template.is_active && (
                    <Badge variant='default' className='bg-green-600'>
                      Active
                    </Badge>
                  )}
                </div>
                <div className='flex items-center gap-1'>
                  {!template.is_active && (
                    <Button
                      variant='ghost'
                      size='icon'
                      title='Activate'
                      onClick={() => onActivate(template.id)}
                    >
                      <Power size={14} />
                    </Button>
                  )}
                  <Button
                    variant='ghost'
                    size='icon'
                    title='Edit'
                    onClick={() => onEdit(template.id)}
                  >
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
              </CardHeader>
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}
