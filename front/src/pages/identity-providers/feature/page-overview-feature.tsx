import { useParams, useNavigate } from 'react-router'
import { useDeleteIdentityProvider, useGetIdentityProviders } from '@/api/identity-providers.api'
import { useState, useMemo } from 'react'
import { toast } from 'sonner'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import {
  IDENTITY_PROVIDERS_URL,
  IDENTITY_PROVIDER_CREATE_URL,
} from '@/routes/sub-router/identity-provider.router'
import PageOverview from '../ui/page-overview'
import { Schemas } from '@/api/api.client'
import type { IdentityProviderListItem } from '../types'

export default function PageOverviewFeature() {
  const { realm_name } = useParams<{ realm_name: string }>()
  const navigate = useNavigate()
  const realm = realm_name || 'master'

  const { data: providersData, isLoading } = useGetIdentityProviders({ realm })
  const { mutate: deleteProvider } = useDeleteIdentityProvider()
  const { confirm, ask, close } = useConfirmDeleteAlert()
  const [filters, setFilters] = useState<Filter[]>([])

  const providers = useMemo<IdentityProviderListItem[]>(() => {
    return (providersData?.data ?? []).map((provider: Schemas.IdentityProviderResponse) => ({
      id: provider.alias,
      alias: provider.alias,
      display_name: provider.display_name ?? provider.alias,
      provider_id: provider.provider_id,
      enabled: provider.enabled,
      updated_at: null,
    }))
  }, [providersData])

  const filterFields: FilterFieldsConfig = [
    {
      key: 'display_name',
      label: 'Name',
      type: 'text',
    },
    {
      key: 'alias',
      label: 'Alias',
      type: 'text',
    },
    {
      key: 'provider_id',
      label: 'Type',
      type: 'text',
    },
    {
      key: 'enabled',
      label: 'Status',
      type: 'boolean',
    },
  ]

  const filteredData = useMemo(() => {
    if (filters.length === 0) return providers

    return providers.filter((provider) => {
      return filters.every((filter) => {
        const fieldValue = provider[filter.field as keyof IdentityProviderListItem]
        const filterValues = filter.values

        switch (filter.operator) {
          case 'is':
            return fieldValue === filterValues[0]
          case 'isNot':
            return fieldValue !== filterValues[0]
          case 'contains':
            return String(fieldValue).toLowerCase().includes(String(filterValues[0]).toLowerCase())
          case 'notContains':
            return !String(fieldValue).toLowerCase().includes(String(filterValues[0]).toLowerCase())
          case 'startsWith':
            return String(fieldValue).toLowerCase().startsWith(String(filterValues[0]).toLowerCase())
          case 'endsWith':
            return String(fieldValue).toLowerCase().endsWith(String(filterValues[0]).toLowerCase())
          case 'empty':
            return !fieldValue || fieldValue === ''
          case 'notEmpty':
            return fieldValue && fieldValue !== ''
          default:
            return true
        }
      })
    })
  }, [providers, filters])

  const statistics = useMemo(() => {
    const uniqueTypes = new Set(providers.map((p) => p.provider_id))
    return {
      totalProviders: providers.length,
      enabledProviders: providers.filter((p) => p.enabled).length,
      disabledProviders: providers.filter((p) => !p.enabled).length,
      providerTypes: uniqueTypes.size,
    }
  }, [providers])

  const handleDeleteSelected = (items: IdentityProviderListItem[]) => {
    items.forEach((item) => {
      deleteProvider(
        {
          path: {
            realm_name: realm,
            alias: item.id,
          },
        },
        {
          onSuccess: () => {
            toast.success(`Provider "${item.display_name}" deleted`)
          },
          onError: () => {
            toast.error(`Failed to delete "${item.display_name}"`)
          },
        }
      )
    })
  }

  const handleCreateProvider = () => {
    navigate(`${IDENTITY_PROVIDERS_URL(realm_name)}${IDENTITY_PROVIDER_CREATE_URL}`)
  }

  const handleDeleteProvider = (providerId: string, providerName: string) => {
    deleteProvider(
      {
        path: {
          realm_name: realm,
          alias: providerId,
        },
      },
      {
        onSuccess: () => {
          toast.success(`Provider "${providerName}" deleted`)
        },
        onError: () => {
          toast.error(`Failed to delete "${providerName}"`)
        },
      }
    )
  }

  const onRowDelete = (provider: IdentityProviderListItem) => {
    ask({
      title: 'Delete provider?',
      description: `Are you sure you want to delete "${provider.display_name}"?`,
      onConfirm: () => {
        handleDeleteProvider(provider.id, provider.display_name)
        close()
      },
    })
  }

  const handleClickRow = (providerId: string) => {
    navigate(`${IDENTITY_PROVIDERS_URL(realm_name)}/${providerId}`)
  }

  return (
    <PageOverview
      data={filteredData}
      isLoading={isLoading}
      realmName={realm}
      statistics={statistics}
      filters={filters}
      filterFields={filterFields}
      onFiltersChange={setFilters}
      confirm={confirm}
      onConfirmClose={close}
      handleDeleteSelected={handleDeleteSelected}
      handleClickRow={handleClickRow}
      handleCreateProvider={handleCreateProvider}
      onRowDelete={onRowDelete}
    />
  )
}
