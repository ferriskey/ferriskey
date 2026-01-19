import { useState, useEffect, useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { toast } from 'sonner'
import { Form } from '@/components/ui/form'
import { useCreateIdentityProvider, type CreateProviderInput } from '@/api/identity-providers.api'
import {
  IDENTITY_PROVIDERS_URL,
  IDENTITY_PROVIDER_OVERVIEW_URL,
} from '@/routes/sub-router/identity-provider.router'
import type { ProviderTemplate } from '@/constants/identity-provider-templates'
import type { ProviderFormData } from '../components/provider-config-form'
import PageCreate from '../ui/page-create'

const formSchema: z.ZodType<ProviderFormData> = z.object({
  displayName: z.string().min(1, 'Display name is required').max(50),
  clientId: z.string().min(1, 'Client ID is required'),
  clientSecret: z.string().min(1, 'Client Secret is required'),
  authorizationUrl: z.string().url('Must be a valid URL').optional().or(z.literal('')),
  tokenUrl: z.string().url('Must be a valid URL').optional().or(z.literal('')),
  userinfoUrl: z.string().url('Must be a valid URL').optional().or(z.literal('')),
  scopes: z.string().optional(),
})

export default function PageCreateFeature() {
  const { realm_name } = useParams<{ realm_name: string }>()
  const navigate = useNavigate()
  const realm = realm_name || 'master'

  const [currentStep, setCurrentStep] = useState(1)
  const [selectedTemplate, setSelectedTemplate] = useState<ProviderTemplate | null>(null)

  const { mutate: createProvider, isPending, data: responseCreateProvider } = useCreateIdentityProvider()

  const form = useForm<ProviderFormData>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      displayName: '',
      clientId: '',
      clientSecret: '',
      authorizationUrl: '',
      tokenUrl: '',
      userinfoUrl: '',
      scopes: '',
    },
    mode: 'onChange',
  })

  const callbackUrl = useMemo(() => {
    const baseUrl = window.location.origin
    const alias = selectedTemplate?.name || 'provider'
    return `${baseUrl}/realms/${realm}/broker/${alias}/endpoint`
  }, [realm, selectedTemplate])

  const url = useMemo(() => {
    if (!realm_name) return ''
    return `${IDENTITY_PROVIDERS_URL(realm_name)}${IDENTITY_PROVIDER_OVERVIEW_URL}`
  }, [realm_name])

  // Pre-fill form when template is selected
  useEffect(() => {
    if (selectedTemplate) {
      form.reset({
        displayName: selectedTemplate.displayName,
        clientId: '',
        clientSecret: '',
        authorizationUrl: selectedTemplate.authorization_url,
        tokenUrl: selectedTemplate.token_url,
        userinfoUrl: selectedTemplate.userinfo_url || '',
        scopes: selectedTemplate.default_scopes.join(' '),
      })
    }
  }, [selectedTemplate, form])

  // Handle successful creation
  useEffect(() => {
    if (responseCreateProvider) {
      toast.success('Identity provider created successfully')
      navigate(url)
    }
  }, [responseCreateProvider, navigate, url])

  const handleSelectTemplate = (template: ProviderTemplate) => {
    setSelectedTemplate(template)
    // Auto-advance to configuration step
    setCurrentStep(2)
  }

  const handleNextStep = () => {
    if (currentStep === 1 && selectedTemplate) {
      setCurrentStep(2)
    } else if (currentStep === 2) {
      // Trigger validation before proceeding
      form.trigger().then((isValid) => {
        if (isValid) {
          setCurrentStep(3)
        }
      })
    }
  }

  const handlePrevStep = () => {
    if (currentStep > 1) {
      setCurrentStep(currentStep - 1)
    }
  }

  const handleSubmit = () => {
    if (!selectedTemplate) return

    const data = form.getValues()

    const config: Record<string, string> = {
      client_id: data.clientId,
      client_secret: data.clientSecret,
      authorization_url: data.authorizationUrl || selectedTemplate.authorization_url,
      token_url: data.tokenUrl || selectedTemplate.token_url,
    }

    if (data.userinfoUrl || selectedTemplate.userinfo_url) {
      config.userinfo_url = data.userinfoUrl || selectedTemplate.userinfo_url || ''
    }

    if (data.scopes) {
      config.scopes = data.scopes
    } else if (selectedTemplate.default_scopes.length > 0) {
      config.scopes = selectedTemplate.default_scopes.join(' ')
    }

    const input: CreateProviderInput = {
      alias: selectedTemplate.name,
      provider_id: selectedTemplate.name,
      display_name: data.displayName || selectedTemplate.displayName,
      enabled: true,
      store_token: false,
      add_read_token_role_on_create: false,
      trust_email: true,
      link_only: false,
      config,
    }

    createProvider({
      path: {
        realm_name: realm,
      },
      body: input,
    })
  }

  const handleCancel = () => {
    navigate(url)
  }

  return (
    <Form {...form}>
      <PageCreate
        form={form}
        currentStep={currentStep}
        selectedTemplate={selectedTemplate}
        callbackUrl={callbackUrl}
        isPending={isPending}
        onSelectTemplate={handleSelectTemplate}
        onNextStep={handleNextStep}
        onPrevStep={handlePrevStep}
        onSubmit={handleSubmit}
        onCancel={handleCancel}
      />
    </Form>
  )
}
