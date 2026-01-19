import { useNavigate, useParams } from 'react-router'
import { useForm } from 'react-hook-form'
import { useEffect, useMemo } from 'react'
import { toast } from 'sonner'
import { Form } from '@/components/ui/form'
import { useIdentityProvider, useUpdateIdentityProvider, useDeleteIdentityProvider } from '@/api/identity-providers.api'
import { IDENTITY_PROVIDERS_URL, IDENTITY_PROVIDER_OVERVIEW_URL } from '@/routes/sub-router/identity-provider.router'
import PageDetail from '../ui/page-detail'

interface UpdateProviderSchema {
  displayName: string
  enabled: boolean
}

export default function PageDetailFeature() {
  const navigate = useNavigate()
  const { realm_name, providerId } = useParams<{ realm_name: string; providerId: string }>()
  const realm = realm_name || 'master'

  const {
    data: provider,
    isLoading,
  } = useIdentityProvider({
    realm,
    providerId: providerId || '',
  })

  const { mutate: updateProvider, data: updateResponse } = useUpdateIdentityProvider()
  const { mutate: deleteProvider } = useDeleteIdentityProvider()

  const form = useForm<UpdateProviderSchema>({
    defaultValues: {
      displayName: '',
      enabled: true,
    },
  })

  const url = useMemo(() => {
    if (!realm_name) return ''
    return `${IDENTITY_PROVIDERS_URL(realm_name)}${IDENTITY_PROVIDER_OVERVIEW_URL}`
  }, [realm_name])

  // Update form when provider loads
  useEffect(() => {
    if (provider) {
      form.reset({
        displayName: provider.display_name ?? '',
        enabled: provider.enabled,
      })
    }
  }, [provider, form])

  // Handle update success
  useEffect(() => {
    if (updateResponse) {
      toast.success('Provider updated successfully')
    }
  }, [updateResponse])

  const handleSubmit = () => {
    const data = form.getValues()
    if (!providerId) return
    updateProvider({
      path: {
        realm_name: realm,
        alias: providerId,
      },
      body: {
        display_name: data.displayName,
        enabled: data.enabled,
      },
    })
  }

  const handleBack = () => {
    navigate(url)
  }

  const handleDelete = () => {
    if (!providerId) return
    deleteProvider(
      {
        path: {
          realm_name: realm,
          alias: providerId,
        },
      },
      {
        onSuccess: () => {
          toast.success('Provider deleted successfully')
          navigate(url)
        },
        onError: () => {
          toast.error('Failed to delete provider')
        },
      }
    )
  }

  const hasChanges = form.formState.isDirty

  return (
    <Form {...form}>
      <PageDetail
        provider={provider || null}
        isLoading={isLoading}
        form={form}
        handleBack={handleBack}
        handleSubmit={handleSubmit}
        handleDelete={handleDelete}
        hasChanges={hasChanges}
      />
    </Form>
  )
}
