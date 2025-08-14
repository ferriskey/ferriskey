import { Permissions } from "@/api/core.interface";
import { useGetClients } from "@/api/client.api";
import { useCreateRole } from "@/api/role.api";
import { RouterParams } from "@/routes/router";
import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { useNavigate, useParams } from "react-router";
import { createRoleSchema, CreateRoleSchema } from "../schemas/create-role.schema";
import { zodResolver } from "@hookform/resolvers/zod";
import { Form } from "@/components/ui/form";
import PageCreateRole from "../ui/page-create-role";
import { ROLE_OVERVIEW_URL, ROLES_URL } from "@/routes/sub-router/role.router";

export default function PageCreateRoleFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()

  const { mutate: createRole, data } = useCreateRole()
  const { data: clientsResponse } = useGetClients({ realm: realm_name || '' })

  const [selectedPermissions, setSelectedPermissions] = useState<Permissions[]>([])

  const handlePermissionToggle = (permission: Permissions) => {

    setSelectedPermissions(prev =>
      prev.includes(permission)
        ? prev.filter(p => p !== permission)
        : [...prev, permission]
    )
  }



  const form = useForm<CreateRoleSchema>({
    resolver: zodResolver(createRoleSchema),
    mode: 'onChange',
    defaultValues: {
      name: '',
      clientId: '',
      description: '',
      permissions: []
    }
  })

  const handleSubmit = () => {
    const values = form.getValues()
    if (!realm_name) return

    createRole({
      body: {
        name: values.name,
        permissions: values.permissions,
        description: values.description
      },
      path: {
        client_id: values.clientId,
        realm_name: realm_name,
      }
    })
  }

  const handleBack = () => {
    navigate(`${ROLES_URL(realm_name)}${ROLE_OVERVIEW_URL}`)
  }

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

  useEffect(() => {
    const li = selectedPermissions.map(perm => perm.toString())
    form.setValue('permissions', li)
  }, [selectedPermissions])

  useEffect(() => {
    if (data) {
      navigate(`${ROLES_URL(realm_name)}${ROLE_OVERVIEW_URL}`)
    }
  }, [data])

  return (
    <Form {...form}>
      <PageCreateRole
        clients={clientsResponse?.data || []}
        form={form}
        handleBack={handleBack}
        handleSubmit={handleSubmit}
        handlePermissionToggle={handlePermissionToggle}
        handleSelectAllInGroup={handleSelectAllInGroup}
        selectedPermissions={selectedPermissions}
      />
    </Form>
  )

}
