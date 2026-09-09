import { useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useCreateRole } from '@/api/role.api'
import { useGetClients } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { createRoleSchema } from '@/pages/role/schemas/create-role.schema'
import PageCreateRole, { type RoleScope } from '@/next/pages/iam/role/ui/page-create-role'
import { CONSOLE_ROLES_URL } from '../urls'

export default function PageCreateRoleFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: clientsResponse } = useGetClients({ realm })
  const { mutate: createRole } = useCreateRole()

  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [scope, setScope] = useState<RoleScope>('realm')
  const [clientId, setClientId] = useState<string>()
  const [permissions, setPermissions] = useState<string[]>([])

  const clients = useMemo(() => clientsResponse?.data ?? [], [clientsResponse])

  const parsed = createRoleSchema.safeParse({
    name,
    description,
    scope,
    clientId,
    permissions,
  })

  const errors = parsed.success
    ? {}
    : {
        name: parsed.error.issues.find((issue) => issue.path[0] === 'name')?.message,
        clientId: parsed.error.issues.find((issue) => issue.path[0] === 'clientId')?.message,
      }

  const handleSubmit = () => {
    if (!parsed.success) return

    createRole(
      {
        realmName: realm,
        clientId: scope === 'client' ? clientId : undefined,
        body: { name, description, permissions },
      },
      { onSuccess: () => navigate(CONSOLE_ROLES_URL(realm)) }
    )
  }

  return (
    <PageCreateRole
      clients={clients}
      name={name}
      description={description}
      scope={scope}
      clientId={clientId}
      permissions={permissions}
      errors={errors}
      canSubmit={parsed.success}
      onNameChange={setName}
      onDescriptionChange={setDescription}
      onScopeChange={setScope}
      onClientIdChange={setClientId}
      onPermissionsChange={setPermissions}
      onBack={() => navigate(CONSOLE_ROLES_URL(realm))}
      onSubmit={handleSubmit}
    />
  )
}
