import { useNavigate, useParams } from 'react-router'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect, useMemo } from 'react'
import { toast } from 'sonner'
import { Form } from '@/components/ui/form'
import { useCreateIdentityProvider, type CreateProviderInput } from '@/api/identity-providers.api'
import { createProviderSchema, type CreateProviderSchema } from '../schemas/create-provider.schema'
import { IDENTITY_PROVIDERS_URL, IDENTITY_PROVIDER_OVERVIEW_URL } from '@/routes/sub-router/identity-provider.router'
import PageCreate from '../ui/page-create'

export default function PageCreateFeature() {
  const { realm_name } = useParams<{ realm_name: string }>()
  const navigate = useNavigate()
  const realm = realm_name || 'master'

  const { mutate: createProvider, data: responseCreateProvider } = useCreateIdentityProvider({
    realm,
  })

  const form = useForm<CreateProviderSchema>({
    resolver: zodResolver(createProviderSchema),
    defaultValues: {
      alias: '',
      displayName: '',
      providerType: 'oidc',
      enabled: true,
      clientId: '',
      clientSecret: '',
      authorizationUrl: '',
      tokenUrl: '',
      userinfoUrl: '',
      entityId: '',
      ssoUrl: '',
      ldapUrl: '',
      bindDn: '',
      bindCredential: '',
    },
  })

  const url = useMemo(() => {
    if (!realm_name) return ''
    return `${IDENTITY_PROVIDERS_URL(realm_name)}${IDENTITY_PROVIDER_OVERVIEW_URL}`
  }, [realm_name])

  const onSubmit = () => {
    const data = form.getValues()

    const config: Record<string, string> = {}

    if (data.providerType === 'oidc' || data.providerType === 'oauth2') {
      if (data.clientId) config.client_id = data.clientId
      if (data.clientSecret) config.client_secret = data.clientSecret
      if (data.authorizationUrl) config.authorization_url = data.authorizationUrl
      if (data.tokenUrl) config.token_url = data.tokenUrl
      if (data.userinfoUrl) config.userinfo_url = data.userinfoUrl
    } else if (data.providerType === 'saml') {
      if (data.entityId) config.entity_id = data.entityId
      if (data.ssoUrl) config.sso_url = data.ssoUrl
    } else if (data.providerType === 'ldap') {
      if (data.ldapUrl) config.ldap_url = data.ldapUrl
      if (data.bindDn) config.bind_dn = data.bindDn
      if (data.bindCredential) config.bind_credential = data.bindCredential
    }

    const input: CreateProviderInput = {
      alias: data.alias,
      display_name: data.displayName,
      provider_type: data.providerType,
      enabled: data.enabled,
      config,
    }

    createProvider(input)
  }

  const handleBack = () => {
    navigate(url)
  }

  useEffect(() => {
    if (responseCreateProvider) {
      toast.success('The identity provider has been successfully created')
      navigate(url)
    }
  }, [responseCreateProvider, navigate, url])

  const formIsValid = form.formState.isValid && form.formState.isDirty

  return (
    <Form {...form}>
      <PageCreate
        form={form}
        handleBack={handleBack}
        handleSubmit={onSubmit}
        formIsValid={formIsValid}
      />
    </Form>
  )
}
