import { useCallback, useEffect, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'

import { useGetClientScope, useUpdateProtocolMapper } from '@/api/client-scope.api'
import { RouterParams } from '@/routes/router'
import {
  CLIENT_SCOPE_MAPPERS_URL,
  CLIENT_SCOPE_URL,
} from '@/routes/sub-router/client-scope.router'
import { ALL_MAPPER_TEMPLATES } from '../constants/protocol-mapper-templates'
import {
  MapperSettingsSchema,
  mapperSettingsSchema,
} from '../schemas/mapper-settings.schema'
import PageProtocolMapperSettings from '../ui/page-protocol-mapper-settings'

// ─── Helpers ──────────────────────────────────────────────────────────────────

/** Convert a mapper.config object (boolean | string | number values) to Record<string, string> */
function configToStrings(config: unknown): Record<string, string> {
  if (!config || typeof config !== 'object') return {}
  return Object.fromEntries(
    Object.entries(config as Record<string, unknown>).map(([k, v]) => [k, String(v)])
  )
}

// ─── Feature ──────────────────────────────────────────────────────────────────

export default function PageProtocolMapperSettingsFeature() {
  const { realm_name, scope_id, mapper_id } = useParams<RouterParams>()
  const navigate = useNavigate()

  const mappersUrl = `${CLIENT_SCOPE_URL(realm_name, scope_id)}${CLIENT_SCOPE_MAPPERS_URL}`

  const { data: scope, isLoading } = useGetClientScope({
    realm: realm_name ?? 'master',
    scopeId: scope_id,
  })

  const { mutate: updateMapper, isPending } = useUpdateProtocolMapper()

  // Resolve mapper from scope data
  const mapper = useMemo(
    () => scope?.protocol_mappers?.find((m) => m.id === mapper_id) ?? null,
    [scope, mapper_id]
  )

  // Match to a template by mapper_type (for dynamic field rendering)
  const template = useMemo(
    () => (mapper ? (ALL_MAPPER_TEMPLATES.find((t) => t.mapper_type === mapper.mapper_type && !t.isCustom) ?? null) : null),
    [mapper]
  )

  // Build initial config values from mapper.config
  const initialConfigValues = useMemo(
    () => (mapper ? configToStrings(mapper.config) : {}),
    // Re-derive only when the mapper id changes, not on every render
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [mapper_id, scope]
  )

  const [configValues, setConfigValues] = useState<Record<string, string>>(initialConfigValues)

  // Sync configValues when mapper data arrives (after initial load)
  useEffect(() => {
    setConfigValues(configToStrings(mapper?.config ?? {}))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [mapper_id, scope])

  // Form – manages only `name` (+config_json fallback for unrecognised mappers)
  const form = useForm<MapperSettingsSchema>({
    resolver: zodResolver(mapperSettingsSchema),
    mode: 'onChange',
    defaultValues: { name: '', config_json: '' },
  })

  // Populate form when mapper loads
  useEffect(() => {
    if (!mapper) return
    form.reset({
      name: mapper.name,
      config_json: mapper.config ? JSON.stringify(mapper.config, null, 2) : '',
    })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [mapper_id, scope])

  // ── Change detection ────────────────────────────────────────────────────────

  const configHasChanges = useMemo(
    () => JSON.stringify(configValues) !== JSON.stringify(initialConfigValues),
    [configValues, initialConfigValues]
  )

  const hasChanges = form.formState.isDirty || configHasChanges

  // ── Handlers ────────────────────────────────────────────────────────────────

  const handleConfigChange = useCallback((key: string, value: string) => {
    setConfigValues((prev) => ({ ...prev, [key]: value }))
  }, [])

  const handleReset = useCallback(() => {
    form.reset()
    setConfigValues(initialConfigValues)
  }, [form, initialConfigValues])

  const handleCancel = useCallback(() => {
    navigate(mappersUrl)
  }, [navigate, mappersUrl])

  const handleSubmit = form.handleSubmit((values) => {
    if (!realm_name || !scope_id || !mapper) return

    let finalConfig: Record<string, unknown>

    if (template) {
      // Build config from dynamic field values
      finalConfig = {}
      for (const [key, val] of Object.entries(configValues)) {
        if (val === 'true') finalConfig[key] = true
        else if (val === 'false') finalConfig[key] = false
        else finalConfig[key] = val
      }
    } else {
      // Fallback: parse raw JSON from form
      finalConfig = {}
      if (values.config_json?.trim()) {
        try {
          finalConfig = JSON.parse(values.config_json) as Record<string, unknown>
        } catch {
          finalConfig = {}
        }
      }
    }

    updateMapper({
      path: { realm_name, scope_id, mapper_id: mapper.id },
      body: {
        name: values.name,
        mapper_type: mapper.mapper_type,
        config: finalConfig,
      },
    })
  })

  // ── Guard: redirect if scope loaded but mapper not found ────────────────────

  if (!isLoading && scope && !mapper) {
    navigate(mappersUrl)
    return null
  }

  return (
    <PageProtocolMapperSettings
      mapper={mapper!}
      template={template}
      form={form}
      configValues={configValues}
      onConfigChange={handleConfigChange}
      hasChanges={hasChanges}
      isPending={isPending}
      onSubmit={handleSubmit}
      onCancel={handleCancel}
      onReset={handleReset}
    />
  )
}
