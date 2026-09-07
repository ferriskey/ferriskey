import { useGetClient } from '@/api/client.api'
import { Form } from '@/components/ui/form'
import { useFormChanges } from '@/hooks/use-form-changes'
import { useSamlServiceProvider } from '@/hooks/use-saml-service-provider'
import { DEFAULT_NAME_ID_FORMAT } from '@/lib/saml'
import { RouterParams } from '@/routes/router'
import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect, useMemo } from 'react'
import { useForm } from 'react-hook-form'
import { useParams } from 'react-router'
import {
  samlServiceProviderSchema,
  SamlServiceProviderSchema,
} from '../schemas/saml-service-provider.schema'
import { isSamlClient } from '@/lib/client-protocol'
import PageClientSaml from '../ui/page-client-saml'

export default function PageClientSamlFeature() {
  const { realm_name, client_id } = useParams<RouterParams>()

  const { data: responseClient } = useGetClient({
    realm: realm_name ?? 'master',
    clientId: client_id,
  })

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
  } = useSamlServiceProvider(realm_name, client_id)

  const values = useMemo<SamlServiceProviderSchema>(
    () => ({
      spEntityId: config?.sp_entity_id ?? '',
      acsUrl: config?.acs_url ?? '',
      nameIdFormat: config?.name_id_format ?? DEFAULT_NAME_ID_FORMAT,
      signAssertions: config?.sign_assertions ?? true,
      signDocuments: config?.sign_documents ?? false,
      wantAuthnRequestsSigned: config?.want_authn_requests_signed ?? false,
    }),
    [config]
  )

  const form = useForm<SamlServiceProviderSchema>({
    resolver: zodResolver(samlServiceProviderSchema),
    defaultValues: values,
  })

  const hasChanges = useFormChanges(form, values)

  useEffect(() => {
    form.reset(values)
  }, [values, form])

  const handleSubmit = form.handleSubmit(async (submitted) => {
    await saveConfig({
      sp_entity_id: submitted.spEntityId.trim(),
      acs_url: submitted.acsUrl.trim(),
      name_id_format: submitted.nameIdFormat,
      sign_assertions: submitted.signAssertions,
      sign_documents: submitted.signDocuments,
      want_authn_requests_signed: submitted.wantAuthnRequestsSigned,
    })
  })

  if (isLoading) {
    return <p className='text-sm text-muted-foreground'>Loading the SAML configuration…</p>
  }

  // The tab is hidden for an OIDC client, but the route stays reachable by URL.
  // Nothing here would ever be served to that client, so nothing is offered.
  if (responseClient && !isSamlClient(responseClient.data.protocol)) {
    return (
      <div className='rounded-lg border bg-muted/30 px-6 py-5'>
        <p className='text-sm font-medium'>This client does not speak SAML</p>
        <p className='text-sm text-muted-foreground mt-1'>
          It was created to speak {responseClient.data.protocol}. A client speaks one
          protocol, chosen when it is created — create a SAML client to configure a service
          provider.
        </p>
      </div>
    )
  }

  return (
    <Form {...form}>
      <PageClientSaml
        form={form}
        isConfigured={isConfigured}
        isSaving={isSavingConfig}
        hasChanges={hasChanges}
        handleSubmit={handleSubmit}
        mapperProps={{
          mappers,
          isCreating: isCreatingMapper,
          isDeleting: isDeletingMapper,
          onAdd: addAttributeMapper,
          onDelete: deleteAttributeMapper,
          onAddCommonProfile: addCommonProfileMappers,
        }}
      />
    </Form>
  )
}
