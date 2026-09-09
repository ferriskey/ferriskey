import { useState } from 'react'
import { Navigate, useNavigate, useParams, useSearchParams } from 'react-router'
import { toast } from 'sonner'
import { useCreateClient } from '@/api/client.api'
import { useUpsertSamlConfig } from '@/api/saml.api'
import { RouterParams } from '@/routes/router'
import { createClientSchema } from '@/pages/client/schemas/create-client.schema'
import { samlServiceProviderSchema } from '@/pages/client/schemas/saml-service-provider.schema'
import { DEFAULT_NAME_ID_FORMAT } from '@/lib/saml'
import { NEXT_CLIENTS_URL } from '@/next/routes'
import { isClientProtocol, type ClientAuthentication } from '../client-choices'
import PageCreateClient, { type CreateClientErrors } from '../ui/page-create-client'

export default function PageCreateClientFeature() {
  const { realm_name } = useParams<RouterParams>()
  const [params] = useSearchParams()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { mutate: createClient } = useCreateClient()
  const { mutateAsync: upsertSamlConfig } = useUpsertSamlConfig()

  const [clientId, setClientId] = useState('')
  const [name, setName] = useState('')
  const [enabled, setEnabled] = useState(true)
  const [authentication, setAuthentication] = useState<ClientAuthentication>('public')
  const [directAccessGrants, setDirectAccessGrants] = useState(false)
  const [deviceCodeGrant, setDeviceCodeGrant] = useState(false)
  const [spEntityId, setSpEntityId] = useState('')
  const [acsUrl, setAcsUrl] = useState('')
  const [nameIdFormat, setNameIdFormat] = useState<string>(DEFAULT_NAME_ID_FORMAT)

  const listUrl = NEXT_CLIENTS_URL(realm)
  const pickerUrl = `${listUrl}?create=1`
  const raw = params.get('protocol')
  const confidential = authentication === 'confidential'

  const identity = createClientSchema.safeParse({
    clientId,
    name,
    enabled,
    clientAuthentication: confidential,
    protocol: raw ?? undefined,
  })

  const saml = samlServiceProviderSchema.safeParse({
    spEntityId,
    acsUrl,
    nameIdFormat,
    signAssertions: true,
    signDocuments: false,
    wantAuthnRequestsSigned: false,
  })

  const isSaml = raw === 'saml'

  const errors: CreateClientErrors = {
    clientId: identity.success
      ? undefined
      : identity.error.issues.find((i) => i.path[0] === 'clientId')?.message,
    name: identity.success
      ? undefined
      : identity.error.issues.find((i) => i.path[0] === 'name')?.message,
    spEntityId:
      !isSaml || saml.success
        ? undefined
        : saml.error.issues.find((i) => i.path[0] === 'spEntityId')?.message,
    acsUrl:
      !isSaml || saml.success
        ? undefined
        : saml.error.issues.find((i) => i.path[0] === 'acsUrl')?.message,
  }

  const canSubmit = identity.success && (!isSaml || saml.success)

  const handleSubmit = () => {
    if (!canSubmit || !isClientProtocol(raw)) return

    createClient(
      {
        path: { realm_name: realm },
        body: {
          client_id: clientId,
          name,
          enabled,
          protocol: raw,
          client_type: confidential ? 'confidential' : 'public',
          public_client: !confidential,
          service_account_enabled: confidential,
          direct_access_grants_enabled: directAccessGrants,
          oauth_device_code_grant_enabled: deviceCodeGrant,
        },
      },
      {
        onSuccess: async (created) => {
          if (raw === 'saml') {
            await upsertSamlConfig({
              realmName: realm,
              clientId: created.id,
              payload: {
                sp_entity_id: spEntityId.trim(),
                acs_url: acsUrl.trim(),
                name_id_format: nameIdFormat,
                sign_assertions: true,
                sign_documents: false,
                want_authn_requests_signed: false,
              },
            })
          }

          toast.success('The client has been successfully created')
          navigate(listUrl)
        },
      }
    )
  }

  if (!isClientProtocol(raw)) return <Navigate to={pickerUrl} replace />

  return (
    <PageCreateClient
      protocol={raw}
      listUrl={listUrl}
      pickerUrl={pickerUrl}
      clientId={clientId}
      name={name}
      enabled={enabled}
      authentication={authentication}
      directAccessGrants={directAccessGrants}
      deviceCodeGrant={deviceCodeGrant}
      spEntityId={spEntityId}
      acsUrl={acsUrl}
      nameIdFormat={nameIdFormat}
      errors={errors}
      canSubmit={canSubmit}
      onClientIdChange={setClientId}
      onNameChange={setName}
      onEnabledChange={setEnabled}
      onAuthenticationChange={setAuthentication}
      onDirectAccessGrantsChange={setDirectAccessGrants}
      onDeviceCodeGrantChange={setDeviceCodeGrant}
      onSpEntityIdChange={setSpEntityId}
      onAcsUrlChange={setAcsUrl}
      onNameIdFormatChange={setNameIdFormat}
      onBack={() => navigate(listUrl)}
      onSubmit={handleSubmit}
    />
  )
}
