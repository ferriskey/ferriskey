import { Role } from '@/api/core.interface'
import RoleMappingModalFeature from '../feature/modals/role-mapping-modal-feature'
import { DataTable } from '@/components/ui/data-table'
import { columns } from '../columns/list-user-roles.column'
import { Delete } from 'lucide-react'

interface PageUserRoleMappingProps {
  userRoles: Role[]
  isLoading: boolean
  isError: boolean
  handleUnassignRole: (roleId: string) => void
  userId?: string
}

export default function PageUserRoleMapping({
  userRoles,
  isLoading,
  handleUnassignRole,
}: PageUserRoleMappingProps) {
  return (
    <div className="">
      <DataTable
        columns={columns}
        data={userRoles ?? []}
        isLoading={isLoading}
        emptyState={<NoUserRoles />}
        rowActions={[
          {
            label: 'Unassign',
            icon: <Delete className="h-4 w-4" />,
            onClick: (role) => {
              handleUnassignRole(role.id)
            },
          },
        ]}
      />
    </div>
  )
}

function NoUserRoles() {
  return (
    <div className="flex flex-col items-center justify-center gap-4 p-8 text-center">
      <div className="w-24 h-24">
        <img src="/icons/cadenas.png" alt="" />
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
