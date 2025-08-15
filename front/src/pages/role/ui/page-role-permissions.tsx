import BadgeColor, { BadgeColorScheme } from '@/components/ui/badge-color.tsx'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { cn } from '@/lib/utils.ts'
import { permissionGroups } from '@/pages/role/types/permission-groups.ts'
import { CheckIcon, LockKeyholeIcon, Shield, Users } from 'lucide-react'
import { useFormContext } from 'react-hook-form'
import FloatingActionBar from '../../../components/ui/floating-action-bar'
import { UpdateRolePermissionsSchema } from '../schemas/update-role.schema'
import { Schemas } from '@/api/api.client'
import Role = Schemas.Role

export interface PageRolePermissionsProps {
  role: Role
  togglePermission: (permission: string) => void
  handleSubmit: () => void
}

export default function PageRolePermissions(props: PageRolePermissionsProps) {
  const { role, togglePermission, handleSubmit } = props

  const { watch } = useFormContext<UpdateRolePermissionsSchema>()
  const permissions = watch('permissions')

  const hasDifferentPermissions = permissions.length !== role.permissions.length
    || !permissions.every((permission) => role.permissions.includes(permission))

  function normalizePermissionName(permission: string) {
    return permission
      .toString()
      .split('_')
      .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
      .join(' ')
  }

  return (
    <div className='space-y-6'>
      <Card className='rounded-sm'>
        <CardHeader className='pb-3'>
          <div className='flex items-center gap-2'>
            <div className='flex items-center justify-center h-10 w-10 rounded-full bg-primary/10'>
              <Shield className='h-5 w-5 text-primary' />
            </div>
            <CardTitle className='text-xl font-bold'>{role.name}</CardTitle>
          </div>
          <CardDescription className='mt-1'>
            {role.description || 'No description provided'}
          </CardDescription>
        </CardHeader>
      </Card>

      <div className='grid sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4'>
        {Object.entries(permissionGroups).map(([groupName, groupPermissions]) => {
          const numberOfPermissions = groupPermissions.length
          const numberOfEnabledPermissions = groupPermissions.filter((permission) =>
            permissions.includes(permission.toString())
          ).length
          return (
            <div key={groupName} className='border rounded-md p-4 shadow'>
              <div className='flex items-center gap-4'>
                <div className='rounded-md bg-primary/20 p-2'>
                  <Users className='text-primary' />
                </div>
                <div>
                  <h3 className='font-bold '>{groupName}</h3>
                  <span className='text-muted-foreground text-sm'>
                    {numberOfEnabledPermissions} of {numberOfPermissions} permissions enabled
                  </span>
                </div>
              </div>
              <div className='flex flex-col gap-3 mt-4'>
                {groupPermissions.map((permission) => {
                  const inRolePermissions = permissions.includes(permission.toString())
                  return (
                    <div
                      className={cn(
                        'p-2 border rounded-sm cursor-pointer',
                        inRolePermissions
                          ? 'bg-primary/10 hover:bg-primary/20'
                          : 'bg-muted/10 hover:bg-muted/20'
                      )}
                      key={permission}
                      onClick={() => togglePermission(permission)}
                    >
                      <div className='flex items-center gap-2'>
                        <span
                          className={cn(
                            'text-sm',
                            inRolePermissions
                              ? 'text-primary font-semibold'
                              : 'text-muted-foreground'
                          )}
                        >
                          {normalizePermissionName(permission)}
                        </span>
                        <div>
                          <BadgeColor
                            color={
                              inRolePermissions ? BadgeColorScheme.PRIMARY : BadgeColorScheme.GRAY
                            }
                          >
                            {inRolePermissions ? 'Enabled' : 'Disabled'}
                          </BadgeColor>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            </div>
          )
        })}
      </div>
      <FloatingActionBar
        title='Update permissions'
        show={hasDifferentPermissions}
        actions={[
          {
            icon: <CheckIcon className='h-4 w-4' />,
            label: 'Submit changes',
            onClick: handleSubmit,
          }
        ]}
        description='Update the role permissions from the selected permissions.'
        icon={<LockKeyholeIcon className='h-4 w-4' />}
      />
    </div>
  )
}
