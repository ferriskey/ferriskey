import { DataTable } from '@/components/ui/data-table'
import { columns } from '../columns/list-client-roles.column'
import { Schemas } from '@/api/api.client'
import { Trash2 } from 'lucide-react'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'

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
  if (isLoading) {
    return <div>Loading roles...</div>
  }

  if (isError) {
    return <div>Error while loading roles.</div>
  }

  function onRowDelete(role: Role) {
    ask({
      title: 'Delete role?',
      description: `Are you sure you want to delete "${role.name}"?`,
      onConfirm: () => {
        if (typeof handleDeleteRole === 'function') {
          handleDeleteRole(role)
        }
        close()
      },
    })
  }

  return (
    <>
      <DataTable
        data={roles}
        columns={columns}
        rowActions={[
          {
            label: 'Delete',
            icon: <Trash2 className='h-4 w-4' />,
            variant: 'destructive',
            onClick: onRowDelete,
          },
        ]}
      />
      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </>
  )
}
