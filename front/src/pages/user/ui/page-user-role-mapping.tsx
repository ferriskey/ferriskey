import { Role } from '@/api/api.interface'
import { Card, CardContent } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import RoleMappingModalFeature from '../feature/modals/role-mapping-modal-feature'

interface PageUserRoleMappingProps {
  userRoles: Role[]
  isLoading: boolean
  isError: boolean
  userId?: string
}

export default function PageUserRoleMapping({
  userRoles,
  isLoading,
  isError,
}: PageUserRoleMappingProps) {
  if (isLoading) {
    return <div>Loading user roles...</div>
  }

  if (isError) {
    return <div>Error loading user roles.</div>
  }

  return (
    <div className="">
      <Card>
        <CardContent>
          {userRoles.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Role Name</TableHead>
                  <TableHead>Description</TableHead>
                  <TableHead>Client</TableHead>
                  <TableHead>Created At</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {userRoles.map((role) => (
                  <TableRow key={role.id}>
                    <TableCell>{role.name}</TableCell>
                    <TableCell>{role.description || '-'}</TableCell>
                    <TableCell>{role.client?.name || '-'}</TableCell>
                    <TableCell>{new Date(role.created_at).toLocaleDateString()}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <NoUserRoles />
          )}
        </CardContent>
      </Card>
    </div>
  )
}

function NoUserRoles() {
  return (
    <div className="flex flex-col items-center justify-center gap-4 p-8 text-center">
      <div className="w-24 h-24">
        <img src="/test.svg" alt="" />
      </div>

      <div className="flex flex-col gap-6">
        <div className="flex flex-col gap-1 w-2/3 mx-auto">
          <span className="text-lg">The user has no roles</span>
          <span className="text-muted-foreground text-sm">
            A role is composed of various permissions. Roles are a convenient way to manage access
            for users.
          </span>
        </div>

        <div>
          <RoleMappingModalFeature />
        </div>
      </div>
    </div>
  )
}
