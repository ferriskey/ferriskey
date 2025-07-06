import { Role } from '@/api/api.interface'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { DataTable } from '@/components/ui/data-table'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { columns } from '../columns/list-client-roles.column'

interface PageClientRolesProps {
  roles: Role[]
  isLoading: boolean
  isError: boolean
  clientId?: string
}

export default function PageClientRoles({
  roles,
  isLoading,
  isError,
  clientId,
}: PageClientRolesProps) {
  if (isLoading) {
    return <div>Loading roles...</div>
  }

  if (isError) {
    return <div>Error loading roles.</div>
  }

  return (
    <div className="">
      <DataTable data={roles} columns={columns} />
    </div>
  )
}
