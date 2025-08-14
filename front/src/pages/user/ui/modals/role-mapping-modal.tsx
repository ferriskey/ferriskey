import { Button } from '@/components/ui/button'
import { DataTable } from '@/components/ui/data-table'
import { Dialog, DialogContent, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Dispatch, SetStateAction } from 'react'
import { columns } from '../../columns/role-mapping.column'
import { FormField } from '@/components/ui/form'
import { UseFormReturn } from 'react-hook-form'
import { AssignRoleSchema } from '../../schemas/assign-role.schema'
import { Schemas } from '@/api/api.client'
import User = Schemas.User
import Role = Schemas.Role

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
                  emptyState={
                    <div className="flex flex-col items-center justify-center gap-4 p-8 text-center">
                      <div className="w-32 h-32">
                        <img src="/icons/cadenas.png" alt="" />
                      </div>

                      <span className="text-lg">No role is available</span>
                    </div>
                  }
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
