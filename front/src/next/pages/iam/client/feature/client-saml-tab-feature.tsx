import { useState } from 'react'
import { useSamlServiceProvider } from '@/hooks/use-saml-service-provider'
import {
  samlAttributeMapperSchema,
  samlServiceProviderSchema,
} from '@/pages/client/schemas/saml-service-provider.schema'
import {
  BUILT_IN_SOURCE_OPTIONS,
  COMMON_PROFILE_MAPPERS,
  CUSTOM_ATTRIBUTE_SOURCE,
  DEFAULT_ATTRIBUTE_NAME_FORMAT,
  DEFAULT_NAME_ID_FORMAT,
  toCustomAttributeSource,
} from '@/lib/saml'
import { Schemas } from '@/api/api.client'
import ClientSamlTab, { type SamlDraft, type SamlMapperDraft } from '../ui/client-saml-tab'

import Client = Schemas.Client

interface Draft extends SamlDraft {
  key: string
}

const EMPTY_MAPPER_DRAFT: SamlMapperDraft = {
  name: '',
  source: BUILT_IN_SOURCE_OPTIONS[0].value,
  customKey: '',
  nameFormat: DEFAULT_ATTRIBUTE_NAME_FORMAT,
}

export interface ClientSamlTabFeatureProps {
  client: Client
  realm: string
}

export default function ClientSamlTabFeature({ client, realm }: ClientSamlTabFeatureProps) {
  const {
    config,
    mappers,
    isConfigured,
    isLoading,
    isSavingConfig,
    isCreatingMapper,
    isDeletingMapper,
    saveConfig,
    addAttributeMapper,
    deleteAttributeMapper,
    addCommonProfileMappers,
  } = useSamlServiceProvider(realm, client.id)

  const pristine: Draft = {
    key: `${client.id}:${config?.updated_at ?? 'none'}`,
    spEntityId: config?.sp_entity_id ?? '',
    acsUrl: config?.acs_url ?? '',
    nameIdFormat: config?.name_id_format ?? DEFAULT_NAME_ID_FORMAT,
    signAssertions: config?.sign_assertions ?? true,
    signDocuments: config?.sign_documents ?? false,
    wantAuthnRequestsSigned: config?.want_authn_requests_signed ?? false,
  }

  const [draft, setDraft] = useState<Draft>(pristine)
  const [mapperDraft, setMapperDraft] = useState<SamlMapperDraft>(EMPTY_MAPPER_DRAFT)

  if (draft.key !== pristine.key) setDraft(pristine)

  const current = draft.key === pristine.key ? draft : pristine

  const parsed = samlServiceProviderSchema.safeParse(current)
  const errors = parsed.success
    ? {}
    : {
        spEntityId: parsed.error.issues.find((i) => i.path[0] === 'spEntityId')?.message,
        acsUrl: parsed.error.issues.find((i) => i.path[0] === 'acsUrl')?.message,
        nameIdFormat: parsed.error.issues.find((i) => i.path[0] === 'nameIdFormat')?.message,
      }

  const parsedMapper = samlAttributeMapperSchema.safeParse(mapperDraft)
  const mapperErrors = parsedMapper.success
    ? {}
    : {
        name: parsedMapper.error.issues.find((i) => i.path[0] === 'name')?.message,
        customKey: parsedMapper.error.issues.find((i) => i.path[0] === 'customKey')?.message,
      }

  const dirtyCount = (Object.keys(pristine) as (keyof Draft)[]).filter(
    (key) => key !== 'key' && current[key] !== pristine[key]
  ).length

  const handleSubmit = async () => {
    if (!parsed.success) return

    await saveConfig({
      sp_entity_id: current.spEntityId.trim(),
      acs_url: current.acsUrl.trim(),
      name_id_format: current.nameIdFormat,
      sign_assertions: current.signAssertions,
      sign_documents: current.signDocuments,
      want_authn_requests_signed: current.wantAuthnRequestsSigned,
    })
  }

  const handleAddMapper = async () => {
    if (!parsedMapper.success) return

    const added = await addAttributeMapper({
      name: mapperDraft.name,
      source:
        mapperDraft.source === CUSTOM_ATTRIBUTE_SOURCE
          ? toCustomAttributeSource(mapperDraft.customKey)
          : mapperDraft.source,
      nameFormat: mapperDraft.nameFormat,
    })

    if (added) {
      setMapperDraft((d) => ({ ...d, name: '', customKey: '' }))
    }
  }

  return (
    <ClientSamlTab
      draft={current}
      errors={errors}
      mapperDraft={mapperDraft}
      mapperErrors={mapperErrors}
      mappers={mappers}
      isConfigured={isConfigured}
      isLoading={isLoading}
      isSaving={isSavingConfig}
      isCreatingMapper={isCreatingMapper}
      isDeletingMapper={isDeletingMapper}
      dirtyCount={dirtyCount}
      canAddMapper={parsedMapper.success}
      onDraftChange={(patch) => setDraft((d) => ({ ...d, ...patch }))}
      onMapperDraftChange={(patch) => setMapperDraft((d) => ({ ...d, ...patch }))}
      onDiscard={() => setDraft(pristine)}
      onSubmit={() => void handleSubmit()}
      onAddMapper={() => void handleAddMapper()}
      onAddCommonProfile={() => void addCommonProfileMappers(COMMON_PROFILE_MAPPERS)}
      onDeleteMapper={(mapperId) => void deleteAttributeMapper(mapperId)}
    />
  )
}
