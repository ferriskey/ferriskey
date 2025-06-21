import { Role, User } from '@/api/api.interface'
import BadgeColor, { BadgeColorScheme } from '@/components/ui/badge-color'
import { Button } from '@/components/ui/button'
import { DataTable } from '@/components/ui/data-table'
import { Dialog, DialogContent, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Dispatch, SetStateAction } from 'react'
import { columns } from '../../columns/role-mapping.column'
import { FormField } from '@/components/ui/form'
import { UseFormReturn } from 'react-hook-form'
import { AssignRoleSchema } from '../../schemas/assign-role.schema'

export interface RoleMappingModalProps {
  open: boolean
  setOpen: Dispatch<SetStateAction<boolean>>
  roles: Role[]
  user: User
  form: UseFormReturn<AssignRoleSchema>
  isValid: boolean
  handleSubmit: () => void
}

export default function RoleMappingModal({
  open,
  setOpen,
  roles,
  user,
  form,
  isValid,
  handleSubmit,
}: RoleMappingModalProps) {
  console.log('RoleMappingModal roles:', roles)

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="outline">Add a role</Button>
      </DialogTrigger>

      <DialogContent className="!max-w-4xl">
        <DialogTitle>Assign roles to {user.username}</DialogTitle>

        <form onSubmit={handleSubmit}>
          <div className="flex flex-col gap-4">
            <FormField
              control={form.control}
              name="roleIds"
              render={({ field }) => (
                <DataTable
                  onSelectionChange={(e) => {
                    const ids = e.map((role) => role.id)
                    field.onChange(ids)
                  }}
                  columns={columns}
                  enableSelection
                  data={roles}
                />
              )}
            />

            <div className="mt-4 flex gap-4">
              <Button disabled={!isValid}>Assign</Button>

              <Button variant="outline" onClick={() => setOpen(false)}>
                Cancel
              </Button>
            </div>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  )
}
