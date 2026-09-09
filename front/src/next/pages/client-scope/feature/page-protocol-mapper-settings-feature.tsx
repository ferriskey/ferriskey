import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useGetClientScope, useUpdateProtocolMapper } from '@/api/client-scope.api'
import { RouterParams } from '@/routes/router'
import { mapperSettingsSchema } from '@/pages/client-scope/schemas/mapper-settings.schema'
import { clientScopeMappersUrl } from '../urls'
import { configFromStrings, configToStrings, parseJsonConfig } from '../config-values'
import { templateForType } from '../mapper-templates'
import PageProtocolMapperSettings from '../ui/page-protocol-mapper-settings'
import { useMapperEntityOptions } from './use-mapper-entity-options'

interface Draft {
  key: string
  name: string
  configJson: string
  config: Record<string, string>
}

const EMPTY_DRAFT: Draft = { key: '', name: '', configJson: '', config: {} }

export default function PageProtocolMapperSettingsFeature() {
  const { realm_name, scope_id, mapper_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: scope, isLoading } = useGetClientScope({ realm, scopeId: scope_id })
  const { mutate: updateProtocolMapper, isPending } = useUpdateProtocolMapper()
  const entityOptions = useMapperEntityOptions(realm)

  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT)

  const mapper = scope?.protocol_mappers?.find((m) => m.id === mapper_id)
  const template = mapper ? templateForType(mapper.mapper_type) : null
  const mappersUrl = clientScopeMappersUrl(realm, scope_id ?? '')

  const pristine: Draft = mapper
    ? {
        key: mapper.id,
        name: mapper.name,
        configJson: mapper.config ? JSON.stringify(mapper.config, null, 2) : '',
        config: configToStrings(mapper.config),
      }
    : EMPTY_DRAFT

  if (mapper && draft.key !== mapper.id) setDraft(pristine)

  const current = mapper && draft.key === mapper.id ? draft : pristine

  const parsed = mapperSettingsSchema.safeParse({
    name: current.name,
    config_json: current.configJson,
  })
  const nameError = parsed.success
    ? undefined
    : parsed.error.issues.find((i) => i.path[0] === 'name')?.message
  const configError = parsed.success
    ? undefined
    : parsed.error.issues.find((i) => i.path[0] === 'config_json')?.message

  const hasChanges =
    current.name !== pristine.name ||
    current.configJson !== pristine.configJson ||
    JSON.stringify(current.config) !== JSON.stringify(pristine.config)

  const handleSubmit = () => {
    if (!scope_id || !mapper || !parsed.success || isPending) return

    updateProtocolMapper({
      path: { realm_name: realm, scope_id, mapper_id: mapper.id },
      body: {
        name: current.name,
        mapper_type: mapper.mapper_type,
        config:
          template && template.fields.length > 0
            ? configFromStrings(current.config)
            : parseJsonConfig(current.configJson),
      },
    })
  }

  return (
    <PageProtocolMapperSettings
      mapper={mapper}
      isLoading={isLoading}
      template={template}
      name={current.name}
      configJson={current.configJson}
      configValues={current.config}
      entityOptions={entityOptions}
      nameError={nameError}
      configError={configError}
      hasChanges={hasChanges}
      isPending={isPending}
      onNameChange={(v) => setDraft((d) => ({ ...d, name: v }))}
      onConfigJsonChange={(v) => setDraft((d) => ({ ...d, configJson: v }))}
      onConfigChange={(key, value) =>
        setDraft((d) => ({ ...d, config: { ...d.config, [key]: value } }))
      }
      onBack={() => navigate(mappersUrl)}
      onReset={() => setDraft(pristine)}
      onSubmit={handleSubmit}
    />
  )
}
