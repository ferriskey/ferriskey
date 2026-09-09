import { useState } from 'react'
import { Navigate, useNavigate, useParams, useSearchParams } from 'react-router'
import { useCreateProtocolMapper } from '@/api/client-scope.api'
import { RouterParams } from '@/routes/router'
import { mapperTemplateFormSchema } from '@/pages/iam/client-scope/schemas/mapper-template-form.schema'
import { clientScopeMappersUrl } from '../urls'
import { configFromStrings, defaultConfigValues, parseJsonConfig } from '../config-values'
import { templateById } from '../mapper-templates'
import PageCreateProtocolMapper from '../ui/page-create-protocol-mapper'
import { useMapperEntityOptions } from './use-mapper-entity-options'

interface Draft {
  key: string
  name: string
  mapperType: string
  configJson: string
  config: Record<string, string>
}

const EMPTY_DRAFT: Draft = { key: '', name: '', mapperType: '', configJson: '', config: {} }

export default function PageCreateProtocolMapperFeature() {
  const { realm_name, scope_id } = useParams<RouterParams>()
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const template = templateById(searchParams.get('template'))
  const mappersUrl = clientScopeMappersUrl(realm, scope_id ?? '')

  const { mutate: createProtocolMapper, isPending } = useCreateProtocolMapper()
  const entityOptions = useMapperEntityOptions(realm)

  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT)

  const pristine: Draft = template
    ? {
        key: template.id,
        name: template.defaultName,
        mapperType: template.isCustom ? '' : template.mapper_type,
        configJson: '',
        config: defaultConfigValues(template.fields),
      }
    : EMPTY_DRAFT

  if (template && draft.key !== template.id) setDraft(pristine)

  const current = template && draft.key === template.id ? draft : pristine

  const parsed = mapperTemplateFormSchema.safeParse({
    name: current.name,
    mapper_type: current.mapperType,
    config_json: current.configJson,
  })

  if (!template) return <Navigate to={mappersUrl} replace />

  const nameError = parsed.success
    ? undefined
    : parsed.error.issues.find((i) => i.path[0] === 'name')?.message
  const configError = parsed.success
    ? undefined
    : parsed.error.issues.find((i) => i.path[0] === 'config_json')?.message

  const typeIsSet = template.isCustom ? current.mapperType.trim() !== '' : true

  const handleSubmit = () => {
    if (!scope_id || !parsed.success || !typeIsSet || isPending) return

    createProtocolMapper(
      {
        path: { realm_name: realm, scope_id },
        body: {
          name: current.name,
          mapper_type: template.isCustom ? current.mapperType : template.mapper_type,
          config: template.isCustom
            ? parseJsonConfig(current.configJson)
            : configFromStrings(current.config),
        },
      },
      { onSuccess: () => navigate(mappersUrl) }
    )
  }

  return (
    <PageCreateProtocolMapper
      template={template}
      name={current.name}
      mapperType={current.mapperType}
      configJson={current.configJson}
      configValues={current.config}
      entityOptions={entityOptions}
      nameError={nameError}
      configError={configError}
      canSubmit={parsed.success && typeIsSet && !isPending}
      isPending={isPending}
      onNameChange={(v) => setDraft((d) => ({ ...d, name: v }))}
      onMapperTypeChange={(v) => setDraft((d) => ({ ...d, mapperType: v }))}
      onConfigJsonChange={(v) => setDraft((d) => ({ ...d, configJson: v }))}
      onConfigChange={(key, value) =>
        setDraft((d) => ({ ...d, config: { ...d.config, [key]: value } }))
      }
      onCancel={() => navigate(mappersUrl)}
      onSubmit={handleSubmit}
    />
  )
}
