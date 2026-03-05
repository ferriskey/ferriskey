import { useCreateProtocolMapper } from '@/api/client-scope.api'
import { RouterParams } from '@/routes/router'
import {
  CLIENT_SCOPE_MAPPERS_URL,
  CLIENT_SCOPE_URL,
} from '@/routes/sub-router/client-scope.router'
import { zodResolver } from '@hookform/resolvers/zod'
import { useCallback, useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useNavigate, useParams, useSearchParams } from 'react-router'
import { ALL_MAPPER_TEMPLATES } from '../constants/protocol-mapper-templates'
import {
  MapperTemplateFormSchema,
  mapperTemplateFormSchema,
} from '../schemas/mapper-template-form.schema'
import PageCreateProtocolMapper from '../ui/page-create-protocol-mapper'

export default function PageCreateProtocolMapperFeature() {
  const { realm_name, scope_id } = useParams<RouterParams>()
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()

  const templateId = searchParams.get('template')
  const template = ALL_MAPPER_TEMPLATES.find((t) => t.id === templateId) ?? null

  const [configValues, setConfigValues] = useState<Record<string, string>>({})

  const { mutate: createMapper, isPending } = useCreateProtocolMapper()

  const form = useForm<MapperTemplateFormSchema>({
    resolver: zodResolver(mapperTemplateFormSchema),
    mode: 'onChange',
    defaultValues: { name: '', mapper_type: '', config_json: '' },
  })

  // Pre-populate defaults when template is resolved
  useEffect(() => {
    if (!template) return

    const defaults: Record<string, string> = {}
    template.fields.forEach((f) => {
      defaults[f.key] = f.defaultValue ?? ''
    })
    setConfigValues(defaults)

    form.reset({
      name: template.defaultName,
      mapper_type: template.isCustom ? '' : template.mapper_type,
      config_json: '',
    })
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [templateId])

  const mappersUrl = `${CLIENT_SCOPE_URL(realm_name, scope_id)}${CLIENT_SCOPE_MAPPERS_URL}`

  const handleCancel = useCallback(() => {
    navigate(mappersUrl)
  }, [navigate, mappersUrl])

  const handleConfigChange = useCallback((key: string, value: string) => {
    setConfigValues((prev) => ({ ...prev, [key]: value }))
  }, [])

  const handleSubmit = form.handleSubmit((values) => {
    if (!realm_name || !scope_id || !template) return

    const resolvedMapperType = template.isCustom
      ? (values.mapper_type ?? '')
      : template.mapper_type

    let finalConfig: Record<string, unknown>

    if (template.isCustom) {
      if (values.config_json && values.config_json.trim()) {
        try {
          finalConfig = JSON.parse(values.config_json) as Record<string, unknown>
        } catch {
          finalConfig = {}
        }
      } else {
        finalConfig = {}
      }
    } else {
      finalConfig = {}
      for (const [key, val] of Object.entries(configValues)) {
        if (val === 'true') finalConfig[key] = true
        else if (val === 'false') finalConfig[key] = false
        else finalConfig[key] = val
      }
    }

    createMapper(
      {
        path: { realm_name, scope_id },
        body: {
          name: values.name,
          mapper_type: resolvedMapperType,
          config: finalConfig,
        },
      },
      {
        onSuccess: () => navigate(mappersUrl),
      }
    )
  })

  // Redirect if template ID is unknown
  if (!template) {
    navigate(mappersUrl)
    return null
  }

  const isValid = form.formState.isValid && !isPending

  return (
    <PageCreateProtocolMapper
      template={template}
      form={form}
      configValues={configValues}
      onConfigChange={handleConfigChange}
      isValid={isValid}
      isPending={isPending}
      onSubmit={handleSubmit}
      onCancel={handleCancel}
    />
  )
}
