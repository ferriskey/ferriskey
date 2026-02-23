import { Schemas } from '@/api/api.client'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { OverviewList } from '@/components/ui/overview-list'
import { EntityAvatar } from '@/components/ui/entity-avatar'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'

import Role = Schemas.Role

interface PageClientRolesProps {
  roles: Role[]
  isLoading: boolean
  isError: boolean
  clientId?: string
  handleDeleteRole?: (role: Role) => void
}

export default function PageClientRoles({
  roles,
  isLoading,
  isError,
  handleDeleteRole,
}: PageClientRolesProps) {
  const { confirm, ask, close } = useConfirmDeleteAlert()

  if (isError) {
    return (
      <div className='flex items-center justify-center h-24 text-sm text-muted-foreground'>
        Error while loading roles.
      </div>
    )
  }

  function onRowDelete(role: Role) {
    ask({
      title: 'Delete role?',
      description: `Are you sure you want to delete "${role.name}"? This action cannot be undone.`,
      onConfirm: () => {
        if (typeof handleDeleteRole === 'function') {
          handleDeleteRole(role)
        }
        close()
      },
    })
  }

  return (
    <div className='flex flex-col gap-6'>
      <OverviewList
        data={roles}
        isLoading={isLoading}
        searchKeys={['name', 'description']}
        searchPlaceholder='Search roles...'
        title={(n) => `Roles (${n})`}
        emptyLabel='No roles found for this client.'
        renderRow={(role) => (
          <div className='flex items-center justify-between px-8 py-4 border-b last:border-b-0 hover:bg-muted/40 transition-colors'>
            <div className='flex items-center gap-4'>
              <EntityAvatar label={role.name} color='#6366F1' />
              <div>
                <span className='text-base font-medium'>{role.name}</span>
                {role.description && (
                  <div className='text-sm text-muted-foreground mt-0.5'>{role.description}</div>
                )}
              </div>
            </div>
            <Button
              variant='ghost'
              size='icon'
              className='text-muted-foreground hover:text-destructive'
              onClick={() => onRowDelete(role)}
            >
              <Trash2 className='h-4 w-4' />
            </Button>
          </div>
        )}
      />

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </div>
  )
}
