import { useState } from 'react'
import { Navigate, useNavigate, useParams, useSearchParams } from 'react-router'
import { toast } from 'sonner'
import { z } from 'zod'
import { useCreateIdentityProvider } from '@/api/identity-providers.api'
import { createProviderSchema } from '@/pages/iam/identity-providers/schemas/create-provider.schema'
import {
  CUSTOM_PROVIDER_TEMPLATE,
  PROVIDER_TEMPLATES,
  getTemplateById,
  type ProviderTemplate,
} from '@/constants/identity-provider-templates'
import { RouterParams } from '@/routes/router'
import type { ProviderProtocol } from '../provider-status'
import PageCreateProvider, {
  type CreateProviderErrors,
  type CreateProviderValues,
} from '../ui/page-create-provider'
import { useIdentityProvidersBase } from '@/hooks/use-section-base'
import { apiErrorMessage } from '@/lib/api-error'

const configSchema = createProviderSchema.extend({
  displayName: z.string().min(1, 'Display name is required').max(50),
  clientId: z.string().min(1, 'Client ID is required'),
  clientSecret: z.string().min(1, 'Client Secret is required'),
  authorizationUrl: z.string().url('Must be a valid URL'),
  tokenUrl: z.string().url('Must be a valid URL'),
  userinfoUrl: z.string().url('Must be a valid URL').optional().or(z.literal('')),
})

const EMPTY_VALUES: CreateProviderValues = {
  alias: '',
  displayName: '',
  clientId: '',
  clientSecret: '',
  authorizationUrl: '',
  tokenUrl: '',
  userinfoUrl: '',
  scopes: [],
}

interface Draft extends CreateProviderValues {
  key: string
}

const EMPTY_DRAFT: Draft = { key: '', ...EMPTY_VALUES }

const isProtocol = (value: string | null): value is ProviderProtocol =>
  value === 'oidc' || value === 'oauth2' || value === 'saml' || value === 'ldap'

const slugify = (value: string) =>
  value
    .toLowerCase()
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/[^a-z0-9-]+/g, '-')
    .replace(/^-+|-+$/g, '')

export default function PageCreateProviderFeature() {
  const identityProvidersBase = useIdentityProvidersBase()
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const [searchParams, setSearchParams] = useSearchParams()
  const realm = realm_name ?? 'master'

  const { mutate: createProvider, isPending } = useCreateIdentityProvider()

  const templateId = searchParams.get('provider')
  const template = templateId ? (getTemplateById(templateId) ?? null) : null
  const protocolParam = searchParams.get('protocol')
  const protocol = isProtocol(protocolParam) ? protocolParam : template?.provider_type

  const [step, setStep] = useState(template ? 2 : 1)
  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT)

  const templateKey = template?.id ?? ''
  const pristine: Draft = template
    ? {
        key: templateKey,
        alias: template.id === 'custom' ? '' : template.name,
        displayName: template.id === 'custom' ? '' : template.displayName,
        clientId: '',
        clientSecret: '',
        authorizationUrl: template.authorization_url,
        tokenUrl: template.token_url,
        userinfoUrl: template.userinfo_url ?? '',
        scopes: [...template.default_scopes],
      }
    : EMPTY_DRAFT

  if (draft.key !== templateKey) setDraft(pristine)

  const values: CreateProviderValues = draft.key === templateKey ? draft : pristine

  const listUrl = identityProvidersBase

  const parsed = configSchema.safeParse({
    ...values,
    providerType: protocol ?? 'oidc',
  })

  const errors: CreateProviderErrors = {}
  if (!parsed.success) {
    for (const issue of parsed.error.issues) {
      const field = issue.path[0]
      if (typeof field === 'string' && field in EMPTY_VALUES && !(field in errors)) {
        errors[field as keyof CreateProviderValues] = issue.message
      }
    }
  }

  const callbackUrl = `${window.apiUrl}/realms/${realm}/broker/${values.alias || 'provider'}/endpoint`

  const templates = protocol
    ? [
        ...PROVIDER_TEMPLATES.filter((item) => item.provider_type === protocol),
        CUSTOM_PROVIDER_TEMPLATE,
      ]
    : []

  const handleSelectTemplate = (next: ProviderTemplate) => {
    const params = new URLSearchParams(searchParams)
    params.set(
      'protocol',
      next.id === 'custom' ? (protocol ?? next.provider_type) : next.provider_type
    )
    params.set('provider', next.id)
    setSearchParams(params, { replace: true })
    setStep(2)
  }

  const handleNext = () => {
    if (step === 1 && template) setStep(2)
    else if (step === 2 && parsed.success) setStep(3)
  }

  const handleBack = () => {
    if (step === 1) {
      navigate(listUrl)
      return
    }

    if (step === 2) {
      const params = new URLSearchParams(searchParams)
      params.delete('provider')
      setSearchParams(params, { replace: true })
      setStep(1)
      return
    }

    setStep(2)
  }

  const handleSubmit = () => {
    if (!template || !parsed.success) return

    const config: Record<string, string> = {
      client_id: values.clientId,
      client_secret: values.clientSecret,
      authorization_url: values.authorizationUrl,
      token_url: values.tokenUrl,
      scopes: values.scopes.join(' '),
    }

    if (values.userinfoUrl) config.userinfo_url = values.userinfoUrl

    createProvider(
      {
        path: { realm_name: realm },
        body: {
          alias: values.alias,
          provider_id: template.name,
          display_name: values.displayName || template.displayName,
          enabled: true,
          store_token: false,
          add_read_token_role_on_create: false,
          trust_email: true,
          link_only: false,
          config,
        },
      },
      {
        onError: (error) => {
          toast.error(apiErrorMessage(error, 'The identity provider could not be created'))
        },
        onSuccess: () => {
          toast.success('Identity provider created successfully')
          navigate(listUrl)
        },
      }
    )
  }

  if (!protocol) return <Navigate to={listUrl} replace />

  return (
    <PageCreateProvider
      protocol={protocol}
      templates={templates}
      template={template}
      step={step}
      values={values}
      errors={step === 2 ? errors : {}}
      callbackUrl={callbackUrl}
      canContinue={step === 1 ? template !== null : parsed.success}
      isPending={isPending}
      onSelectTemplate={handleSelectTemplate}
      onChange={(patch) =>
        setDraft((current) => ({
          ...current,
          ...patch,
          ...(patch.alias !== undefined ? { alias: slugify(patch.alias) } : {}),
        }))
      }
      onNext={handleNext}
      onBack={handleBack}
      onSubmit={handleSubmit}
    />
  )
}
