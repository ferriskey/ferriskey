import { DataTable } from '@/components/ui/data-table'
import { columns } from '../columns/list-client-roles.column'
import { Schemas } from '@/api/api.client'
import Role = Schemas.Role

interface PageClientRolesProps {
  roles: Role[]
  isLoading: boolean
  isError: boolean
  clientId?: string
  handleDeleteRole?: (role: Role) => void
}

export default function PageClientRoles({ roles, isLoading, isError, handleDeleteRole }: PageClientRolesProps) {
  if (isLoading) {
    return <div>Loading roles...</div>
  }

  if (isError) {
    return <div>Error while loading roles.</div>
  }

  return (
    <div className=''>
      <DataTable
        data={roles}
        columns={columns}
        rowActions={[
          {
            label: 'Delete',
            icon: <span className='text-red-500'>🗑️</span>,
            variant: 'destructive',
            onClick: (role) => handleDeleteRole && handleDeleteRole(role),
          },
        ]}
      />
    </div>
  )
}
