import { useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import {
  useDeleteClientScope,
  useDeleteProtocolMapper,
  useGetClientScope,
  useUpdateClientScope,
} from '@/api/client-scope.api'
import { RouterParams } from '@/routes/router'
import { useRouteTabs } from '@/components/kit'
import { updateClientScopeSchema } from '@/pages/client-scope/schemas/update-client-scope.schema'
import { NEXT_CLIENT_SCOPES_URL } from '@/next/routes'
import { Schemas } from '@/api/api.client'
import { clientScopeMappersUrl, clientScopeUrl, protocolMapperUrl } from '../urls'
import PageClientScopeDetail from '../ui/page-client-scope-detail'
import type { ScopeTypeChoice } from '../ui/page-create-client-scope'

import ProtocolMapper = Schemas.ProtocolMapper

interface Draft {
  key: string
  name: string
  description: string
  scopeType: ScopeTypeChoice
}

const EMPTY_DRAFT: Draft = { key: '', name: '', description: '', scopeType: 'optional' }

const SCOPE_TABS = [
  { key: 'details', label: 'Settings' },
  { key: 'mappers', label: 'Protocol Mappers' },
] as const

export default function PageClientScopeDetailFeature() {
  const { realm_name, scope_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: scope, isLoading } = useGetClientScope({ realm, scopeId: scope_id })
  const { mutate: updateClientScope, isPending } = useUpdateClientScope()
  const { mutate: deleteClientScope } = useDeleteClientScope()
  const { mutate: deleteProtocolMapper } = useDeleteProtocolMapper()

  const base = clientScopeUrl(realm, scope_id ?? '')
  const mappers = useMemo(() => scope?.protocol_mappers ?? [], [scope])
  const tabsWithCount = useMemo(
    () => SCOPE_TABS.map((tab) => (tab.key === 'mappers' ? { ...tab, count: mappers.length } : tab)),
    [mappers.length]
  )
  const { value: tab, tabs } = useRouteTabs(base, tabsWithCount)

  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT)
  const [pickerOpen, setPickerOpen] = useState(false)

  const scopeKey = scope?.id ?? ''
  const pristine: Draft = scope
    ? {
        key: scopeKey,
        name: scope.name,
        description: scope.description ?? '',
        scopeType: scope.default_scope_type === 'DEFAULT' ? 'default' : 'optional',
      }
    : EMPTY_DRAFT

  if (scope && draft.key !== scopeKey) setDraft(pristine)

  const { name, description, scopeType } = draft.key === scopeKey ? draft : pristine

  const parsed = updateClientScopeSchema.safeParse({ name, description, scopeType })
  const nameError = parsed.success
    ? undefined
    : parsed.error.issues.find((i) => i.path[0] === 'name')?.message

  const dirtyCount =
    (name !== pristine.name ? 1 : 0) +
    (description !== pristine.description ? 1 : 0) +
    (scopeType !== pristine.scopeType ? 1 : 0)

  const save = () => {
    if (!scope || !scope_id || !parsed.success || isPending) return

    updateClientScope({
      path: { realm_name: realm, scope_id },
      body: {
        name,
        description: description.trim() || null,
        protocol: scope.protocol,
        is_default: scopeType === 'default',
      },
    })
  }

  const handleDelete = () => {
    if (!scope_id) return
    deleteClientScope(
      { path: { realm_name: realm, scope_id } },
      { onSuccess: () => navigate(NEXT_CLIENT_SCOPES_URL(realm)) }
    )
  }

  return (
    <PageClientScopeDetail
      scope={scope}
      isLoading={isLoading}
      tab={tab}
      tabs={tabs}
      name={name}
      description={description}
      scopeType={scopeType}
      nameError={dirtyCount > 0 ? nameError : undefined}
      dirtyCount={dirtyCount}
      isPending={isPending}
      mapperHref={(mapper: ProtocolMapper) =>
        protocolMapperUrl(realm, scope_id ?? '', mapper.id)
      }
      pickerOpen={pickerOpen}
      onPickerOpenChange={setPickerOpen}
      onSelectTemplate={(templateId: string) => {
        setPickerOpen(false)
        navigate(`${clientScopeMappersUrl(realm, scope_id ?? '')}/new?template=${templateId}`)
      }}
      onDeleteMapper={(mapper: ProtocolMapper) => {
        if (!scope_id) return
        deleteProtocolMapper({
          path: { realm_name: realm, scope_id, mapper_id: mapper.id },
        })
      }}
      onNameChange={(v) => setDraft((d) => ({ ...d, name: v }))}
      onDescriptionChange={(v) => setDraft((d) => ({ ...d, description: v }))}
      onScopeTypeChange={(v) => setDraft((d) => ({ ...d, scopeType: v }))}
      onBack={() => navigate(NEXT_CLIENT_SCOPES_URL(realm))}
      onDiscard={() => setDraft(pristine)}
      onSave={save}
      onDelete={handleDelete}
    />
  )
}
