import { Permissions } from "@/api/api.interface";
import { Badge } from "@/components/ui/badge";
import BadgeColor, { BadgeColorScheme } from "@/components/ui/badge-color";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { InputText } from "@/components/ui/input-text";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { createRoleSchema, CreateRoleSchema } from "../schemas/create-role.schema";
import { zodResolver } from "@hookform/resolvers/zod";

export default function CreateRoleModalFeature() {
  const [selectedPermissions, setSelectedPermissions] = useState<Permissions[]>([]);

  const handlePermissionToggle = (permission: Permissions) => {
    setSelectedPermissions(prev => 
      prev.includes(permission) 
        ? prev.filter(p => p !== permission)
        : [...prev, permission]
    );
  };

  const form = useForm<CreateRoleSchema>({
    resolver: zodResolver(createRoleSchema),
    defaultValues: {
      name: '',
      description: '',
      permissions: []
    }
  })

  const permissionsList = Object.values(Permissions).filter(value => typeof value === 'string');

  const handleSelectAllInGroup = (groupPermissions: Permissions[]) => {
    const allSelected = groupPermissions.every(perm => selectedPermissions.includes(perm))

    if (allSelected) {
      setSelectedPermissions(prev => prev.filter(perm => !groupPermissions.includes(perm)))
    } else {
      setSelectedPermissions(prev => {
        const newPerms = [...prev]
        groupPermissions.forEach(perm => {
          if (!newPerms.includes(perm)) {
            newPerms.push(perm)
          }
        })

        return newPerms
      })
    }
  }

  const permissionGroups = {
    "User Management": [
      Permissions.ManageUsers,
      Permissions.ViewUsers,
      Permissions.QueryUsers,
    ],
    "Client Management": [
      Permissions.CreateClient,
      Permissions.ManageClients,
      Permissions.ViewClients,
      Permissions.QueryClients,
    ],
    "Role & Authorization": [
      Permissions.ManageRoles,
      Permissions.ViewRoles,
      Permissions.ManageAuthorization,
      Permissions.ViewAuthorization,
    ],
    "Realm Management": [
      Permissions.ManageRealm,
      Permissions.ViewRealm,
      Permissions.QueryRealms,
    ],
    "Identity Providers": [
      Permissions.ManageIdentityProviders,
      Permissions.ViewIdentityProviders,
    ],
    "Events & Audit": [
      Permissions.ManageEvents,
      Permissions.ViewEvents,
    ],
    "Groups": [
      Permissions.QueryGroups,
    ],
  }

  const formatPermissionName = (permission: string) => {
    return permission
      .replace(/_/g, ' ')
      .replace(/\b\w/g, l => l.toUpperCase())
  }

  const getPermissionVariant = (permission: Permissions) => {
    if (permission.toString().startsWith('manage')) return BadgeColorScheme.RED;
    if (permission.toString().startsWith('create')) return BadgeColorScheme.GREEN;
    if (permission.toString().startsWith('view')) return BadgeColorScheme.BLUE;
    if (permission.toString().startsWith('query')) return BadgeColorScheme.YELLOW;
    return BadgeColorScheme.GRAY;
  };

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button>
          Create Role
        </Button>
      </DialogTrigger>

      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Create Role</DialogTitle>
          <DialogDescription>
            Fill in the details to create a new role.
          </DialogDescription>
        </DialogHeader>

        <div>
          <div className="flex flex-col gap-4">
            <InputText 
              name="name"
              label="Name"
            />

            <InputText 
              name="description"
              label="Description"
            />

            {/* Permissions sélectionnées */}
            <div>
              <Label className="text-sm font-medium">
                Selected Permissions ({selectedPermissions.length})
              </Label>
              <div className="mt-2 flex flex-wrap gap-1 min-h-[60px] p-2 border rounded-md bg-muted/20">
                {selectedPermissions.length === 0 ? (
                  <span className="text-sm text-muted-foreground">No permissions selected</span>
                ) : (
                  selectedPermissions.map((permission) => (
                    <Badge 
                      key={permission} 
                      variant={getPermissionVariant(permission)}
                      className="text-xs cursor-pointer hover:bg-destructive hover:text-destructive-foreground"
                      onClick={() => handlePermissionToggle(permission)}
                    >
                      {formatPermissionName(permission.toString())}
                      <span className="ml-1">×</span>
                    </Badge>
                  ))
                )}
              </div>
            </div>
          </div>

          {/* Sélection des permissions */}
          <div>
            <Label className="text-sm font-medium mb-3 block">
              Available Permissions
            </Label>
            <ScrollArea className="h-[400px] rounded-md border bg-background">
              <div className="p-4 space-y-4">
                {Object.entries(permissionGroups).map(([groupName, groupPermissions]) => {
                  const allSelected = groupPermissions.every(perm => selectedPermissions.includes(perm));
                  const someSelected = groupPermissions.some(perm => selectedPermissions.includes(perm));
                  
                  return (
                    <div key={groupName} className="space-y-3">
                      {/* En-tête du groupe */}
                      <div className="flex items-center justify-between">
                        <div className="flex items-center space-x-2">
                          <Checkbox
                            id={`group-${groupName}`}
                            checked={allSelected}
                            ref={(el) => {
                              if (el) el.indeterminate = someSelected && !allSelected;
                            }}
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

                      {/* Permissions du groupe */}
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
                              color={getPermissionVariant(permission)}
                              className="text-xs"
                            >
                              {permission.toString().split('_')[0]}
                            </BadgeColor>
                          </div>
                        ))}
                      </div>
                      
                      {Object.keys(permissionGroups).indexOf(groupName) < Object.keys(permissionGroups).length - 1 && (
                        <Separator className="mt-4" />
                      )}
                    </div>
                  );
                })}
              </div>
            </ScrollArea>
          </div>
        </div>

        <DialogFooter>
          <Button type="submit">Save changes</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}