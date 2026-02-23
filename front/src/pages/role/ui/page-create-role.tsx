import { UseFormReturn } from 'react-hook-form'
import { CreateRoleSchema } from '../schemas/create-role.schema'
import { Permissions } from '@/api/core.interface'
import { FormField } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { ArrowLeft, X } from 'lucide-react'
import { Label } from '@/components/ui/label'
import { formatPermissionName } from '@/utils'
import BadgeColor from '@/components/ui/badge-color'
import { BadgeColorScheme } from '@/components/ui/badge-color.enum'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Checkbox } from '@/components/ui/checkbox'
import SelectClientBox from './components/select-client-box'
import { permissionGroups } from '@/pages/role/types/permission-groups.ts'
import { Schemas } from '@/api/api.client.ts'
import Client = Schemas.Client

export interface PageCreateRoleProps {
  form: UseFormReturn<CreateRoleSchema>
  handleSubmit: () => void
  handleBack: () => void
  clients: Client[]
  selectedPermissions: Permissions[]
  handleSelectAllInGroup: (groupPermissions: Permissions[]) => void
  handlePermissionToggle: (permission: Permissions) => void
}

export default function PageCreateRole({
  form,
  handleSubmit,
  handleBack,
  clients,
  selectedPermissions,
  handleSelectAllInGroup,
  handlePermissionToggle,
}: PageCreateRoleProps) {
  const isValid = form.formState.isValid

  return (
    <div className='flex flex-col gap-6'>
      {/* Breadcrumb */}
      <div className='flex items-center gap-2'>
        <button
          onClick={handleBack}
          className='px-4 py-1.5 rounded-md text-sm font-medium transition-colors border bg-transparent text-foreground border-border hover:bg-muted flex items-center gap-2'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Roles
        </button>
        <span className='text-muted-foreground text-sm'>/</span>
        <span className='px-4 py-1.5 rounded-md text-sm font-medium border bg-primary/10 text-primary border-primary/40'>
          New Role
        </span>
      </div>

      {/* Role Details section */}
      <div className='-mx-8 border-t border-b overflow-hidden'>
        <div className='px-8 py-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Role overview</p>
          <h2 className='text-base font-semibold'>Role Details</h2>
        </div>

        {/* Name */}
        <FormField
          control={form.control}
          name='name'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Role Name</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Unique name for this role.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Role Name' {...field} error={form.formState.errors.name?.message} />
              </div>
            </div>
          )}
        />

        {/* Description */}
        <FormField
          control={form.control}
          name='description'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Description</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Optional description for this role.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Description' {...field} error={form.formState.errors.description?.message} />
              </div>
            </div>
          )}
        />

        {/* Client */}
        <FormField
          control={form.control}
          name='clientId'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Client</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Associate this role with a specific client.</p>
              </div>
              <div className='w-1/2'>
                <SelectClientBox clients={clients} onValueChange={field.onChange} />
              </div>
            </div>
          )}
        />

        {/* Permissions */}
        <div className='flex items-start justify-between px-8 py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Permissions</p>
            <p className='text-sm text-muted-foreground mt-0.5'>
              Select the permissions this role grants.
            </p>
            {/* Selected permissions */}
            <div className='flex flex-wrap gap-1 mt-3 min-h-[36px]'>
              {selectedPermissions.length === 0 ? (
                <span className='text-xs text-muted-foreground'>No permissions selected</span>
              ) : (
                selectedPermissions.map((permission, index) => (
                  <div key={index} onClick={() => handlePermissionToggle(permission)}>
                    <BadgeColor
                      color={BadgeColorScheme.PRIMARY}
                      className='text-xs cursor-pointer flex items-center gap-1'
                    >
                      {formatPermissionName(permission.toString())}
                      <X size={11} />
                    </BadgeColor>
                  </div>
                ))
              )}
            </div>
          </div>
          <div className='w-1/2'>
            <ScrollArea className='h-[400px] rounded-md border bg-background'>
              <div className='p-4 space-y-4'>
                {Object.entries(permissionGroups).map(([groupName, groupPermissions]) => {
                  const allSelected = groupPermissions.every(perm => selectedPermissions.includes(perm))
                  return (
                    <div key={groupName} className='space-y-3'>
                      <div className='flex items-center justify-between'>
                        <div className='flex items-center space-x-2'>
                          <Checkbox
                            id={`group-${groupName}`}
                            checked={allSelected}
                            onCheckedChange={() => handleSelectAllInGroup(groupPermissions)}
                          />
                          <Label htmlFor={`group-${groupName}`} className='text-sm font-medium cursor-pointer'>
                            {groupName}
                          </Label>
                        </div>
                        <BadgeColor color={BadgeColorScheme.GRAY} className='text-xs'>
                          {groupPermissions.filter(perm => selectedPermissions.includes(perm)).length}/{groupPermissions.length}
                        </BadgeColor>
                      </div>
                      <div className='ml-6 space-y-2'>
                        {groupPermissions.map((permission) => (
                          <div key={permission} className='flex items-center space-x-2'>
                            <Checkbox
                              id={permission.toString()}
                              checked={selectedPermissions.includes(permission)}
                              onCheckedChange={() => handlePermissionToggle(permission)}
                            />
                            <Label htmlFor={permission.toString()} className='text-sm cursor-pointer flex-1'>
                              {formatPermissionName(permission.toString())}
                            </Label>
                            <BadgeColor color={BadgeColorScheme.PRIMARY} className='text-xs'>
                              {permission.toString().split('_')[0]}
                            </BadgeColor>
                          </div>
                        ))}
                      </div>
                      {Object.keys(permissionGroups).indexOf(groupName) < Object.keys(permissionGroups).length - 1 && (
                        <Separator />
                      )}
                    </div>
                  )
                })}
              </div>
            </ScrollArea>
          </div>
        </div>
      </div>

      <FloatingActionBar
        show={isValid}
        title='Create Role'
        onCancel={handleBack}
        description='Create a new role with the specified permissions.'
        actions={[{ label: 'Create', onClick: handleSubmit }]}
      />
    </div>
  )
}
