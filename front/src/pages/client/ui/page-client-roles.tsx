import { DataTable } from '@/components/ui/data-table'
import { columns } from '../columns/list-client-roles.column'
import { Schemas } from '@/api/api.client'
import Role = Schemas.Role

interface PageClientRolesProps {
  roles: Role[]
  isLoading: boolean
  isError: boolean
  clientId?: string
}

export default function PageClientRoles({ roles, isLoading, isError }: PageClientRolesProps) {
  if (isLoading) {
    return <div>Loading roles...</div>
  }

  if (isError) {
    return <div>Error loading roles.</div>
  }

  return (
    <div className=''>
      <DataTable data={roles} columns={columns} />
    </div>
  )
}
