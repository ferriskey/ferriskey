import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useCreateClientScope } from '@/api/client-scope.api'
import type { CreateClientScopeBody } from '@/api/client-scope-dynamic'
import { RouterParams } from '@/routes/router'
import { createClientScopeSchema } from '@/pages/iam/client-scope/schemas/create-client-scope.schema'
import { CLIENT_SCOPES_URL } from '@/routes/router'
import PageCreateClientScope, { type ScopeTypeChoice } from '../ui/page-create-client-scope'

const PROTOCOL = 'openid-connect'

export default function PageCreateClientScopeFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { mutate: createClientScope, isPending } = useCreateClientScope()

  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [scopeType, setScopeType] = useState<ScopeTypeChoice>('optional')
  const [dynamicAllowed, setDynamicAllowed] = useState(false)

  const parsed = createClientScopeSchema.safeParse({
    name,
    description,
    protocol: PROTOCOL,
    scopeType,
  })

  const nameError = parsed.success
    ? undefined
    : parsed.error.issues.find((i) => i.path[0] === 'name')?.message

  const isDirty = name !== '' || description !== '' || scopeType !== 'optional' || dynamicAllowed

  const handleSubmit = () => {
    if (!parsed.success || isPending) return

    const body: CreateClientScopeBody = {
      name,
      description: description.trim() || null,
      protocol: PROTOCOL,
      is_default: scopeType === 'default',
      dynamic_registration_allowed: dynamicAllowed,
    }

    createClientScope(
      { path: { realm_name: realm }, body },
      { onSuccess: () => navigate(CLIENT_SCOPES_URL(realm)) }
    )
  }

  return (
    <PageCreateClientScope
      name={name}
      description={description}
      protocol={PROTOCOL}
      scopeType={scopeType}
      dynamicAllowed={dynamicAllowed}
      nameError={name ? nameError : undefined}
      canSubmit={parsed.success && isDirty && !isPending}
      isPending={isPending}
      onNameChange={setName}
      onDescriptionChange={setDescription}
      onScopeTypeChange={setScopeType}
      onDynamicAllowedChange={setDynamicAllowed}
      onBack={() => navigate(CLIENT_SCOPES_URL(realm))}
      onSubmit={handleSubmit}
    />
  )
}
