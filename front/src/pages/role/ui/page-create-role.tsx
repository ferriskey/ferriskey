import { UseFormReturn } from "react-hook-form";
import { CreateRoleSchema } from "../schemas/create-role.schema";
import { Client, Permissions } from "@/api/core.interface";
import { FormField } from "@/components/ui/form";
import { InputText } from "@/components/ui/input-text";
import { Button } from "@/components/ui/button";
import { ArrowLeft, Smile, X } from "lucide-react";
import { Label } from "@/components/ui/label";
import { formatPermissionName } from "@/utils";
import BadgeColor, { BadgeColorScheme } from "@/components/ui/badge-color";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Checkbox } from "@/components/ui/checkbox";
import { Heading } from "@/components/ui/heading";
import FloatingActionBar from "@/components/ui/floating-action-bar";
import SelectClientBox from "./components/select-client-box";
import { permissionGroups } from '@/pages/role/types/permission-groups.ts'

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
  handlePermissionToggle 
}: PageCreateRoleProps) {
  const isValid = form.formState.isValid

  return (
    <div className="flex flex-col p-4 gap-4">
      <div className="flex items-center gap-3">

        <Button variant="ghost" size="icon" onClick={handleBack}>
          <ArrowLeft className="h-3 w-3" />
        </Button>
        <span className="text-gray-500 text-sm font-medium">Back to roles</span>
      </div>

      <div className="flex flex-col mb-4">
        <Heading size={3} className="text-gray-800 ">
          Create Role
        </Heading>
        <p className="text-sm text-gray-500 mt-1">
          Define a new role with specific permissions that can be assigned to users or clients.
        </p>
      </div>
      <div className="flex flex-col gap-4 p-4 bg-muted/25 rounded-md border lg:w-2/3">
        <div>
          <Heading size={5} className="text-gray-800">
            Role Details
          </Heading>
        </div>

        <div className="flex flex-col gap-2">
          <FormField 
            control={form.control}
            name="name"
            render={({ field }) => (
              <InputText 
                {...field}
                label="Role Name"
                name="name"
                error={form.formState.errors.name?.message}
              />
            )}
          />

          <FormField 
            control={form.control}
            name="description"
            render={({ field }) => (
              <InputText 
                {...field}
                label="Description"
                name="description"
                error={form.formState.errors.description?.message}
              />
            )}
          />
          <FormField
            control={form.control}
            name="clientId"
            render={({ field }) => (
              <SelectClientBox 
                clients={clients} 
                onValueChange={field.onChange} 
              />
            )}
          />
        </div>       
      </div>

      <div className="flex flex-col p-4 bg-muted/25 rounded-md border lg:w-2/3">
        <div className="flex flex-col gap-2 mb-4">
          <Heading size={5} className="text-gray-800">
            Permissions
          </Heading>

          <p className="text-sm text-gray-500 mb-2">
            Select the permissions that this role will have. You can select multiple permissions from different groups.
          </p>


          <div className="grid grid-cols-6 gap-4 grid-flow-dense">

            <div className="col-span-3 order-2">
              <div className="flex flex-wrap gap-1 min-h-[60px] p-2 border rounded-md bg-white">
                {selectedPermissions.length === 0 ? (
                  <span className="flex w-full items-center justify-center text-neutral-400">No permissions selected</span>
                ) : (
                  selectedPermissions.map((permission) => (
                    <div
                      onClick={() => handlePermissionToggle(permission)}
                    >
                      <BadgeColor 
                        key={permission} 
                        color={BadgeColorScheme.PRIMARY}
                        className="text-xs cursor-pointer flex items-center gap-1"
                      >
                        {formatPermissionName(permission.toString())}
                        <X size={13} />
                      </BadgeColor>
                    </div>
                  ))
                )}
              </div>
            </div>

            <div className="col-span-3">
              <ScrollArea className="h-[500px] rounded-md border bg-background">
                <div className="p-4 space-y-4">
                  {Object.entries(permissionGroups).map(([groupName, groupPermissions]) => {
                    const allSelected = groupPermissions.every(perm => selectedPermissions.includes(perm))
              
                    return (
                      <div key={groupName} className="space-y-3">
                        {/* Header group */}

                        <div className="flex items-center justify-between">
                          <div className="flex items-center space-x-2">
                            <Checkbox 
                              id={`group-${groupName}`}
                              checked={allSelected}
                              onCheckedChange={() => handleSelectAllInGroup(groupPermissions)}
                            />

                            <Label
                              htmlFor={`group-${groupName}`}
                              className="text-sm font-medium cursor-pointer"
                            >
                              {groupName}
                            </Label>
                          </div>

                          <BadgeColor color={BadgeColorScheme.GRAY} className="text-xs">
                            {groupPermissions.filter(perm => selectedPermissions.includes(perm)).length}/{groupPermissions.length}
                          </BadgeColor>
                        </div>

                        {/* Permissions in the group */}

                        <div className="ml-6 space-y-2">
                          {groupPermissions.map((permission) => (
                            <div key={permission} className="flex items-center space-x-2">
                              <Checkbox 
                                id={permission.toString()}
                                checked={selectedPermissions.includes(permission)}
                                onCheckedChange={() => handlePermissionToggle(permission)}
                              />

                              <Label
                                htmlFor={permission.toString()}
                                className="text-sm cursor-pointer flex-1"
                              >
                                {formatPermissionName(permission.toString())}
                              </Label>

                              <BadgeColor
                                color={BadgeColorScheme.PRIMARY}
                                className="text-xs"
                              >
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
          title="Create Role"
          show={isValid}
          actions={[
            {
              label: "Create Role",
              variant: "default",
              onClick: handleSubmit,
            }
          ]}
          description="Once you create this role, it will be available for assignment to users and clients."
          icon={<Smile className="h-4 w-4" />}
        />      
      </div>
    </div>
  )
}
