import { useParams, useNavigate } from 'react-router'
import { useIdentityProviders, useDeleteIdentityProvider, type IdentityProvider } from '@/api/identity-providers.api'
import { useEffect, useState, useMemo } from 'react'
import { toast } from 'sonner'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import {
  IDENTITY_PROVIDERS_URL,
  IDENTITY_PROVIDER_CREATE_URL,
} from '@/routes/sub-router/identity-provider.router'
import PageOverview from '../ui/page-overview'

export default function PageOverviewFeature() {
  const { realm_name } = useParams<{ realm_name: string }>()
  const navigate = useNavigate()
  const realm = realm_name || 'master'

  const { data, isLoading } = useIdentityProviders({ realm })
  const { mutate: deleteProvider, data: responseDeleteProvider } = useDeleteIdentityProvider({
    realm,
    providerId: '',
  })
  const { confirm, ask, close } = useConfirmDeleteAlert()
  const [filters, setFilters] = useState<Filter[]>([])

  const providers = useMemo(() => data || [], [data])

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
      key: 'provider_type',
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
        const fieldValue = provider[filter.field as keyof IdentityProvider]
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
    const uniqueTypes = new Set(providers.map((p) => p.provider_type))
    return {
      totalProviders: providers.length,
      enabledProviders: providers.filter((p) => p.enabled).length,
      disabledProviders: providers.filter((p) => !p.enabled).length,
      providerTypes: uniqueTypes.size,
    }
  }, [providers])

  const handleDeleteSelected = (items: IdentityProvider[]) => {
    items.forEach((item) => {
      deleteProvider(undefined, {
        onSuccess: () => {
          toast.success(`Provider "${item.display_name}" deleted`)
        },
      })
    })
  }

  const handleCreateProvider = () => {
    navigate(`${IDENTITY_PROVIDERS_URL(realm_name)}${IDENTITY_PROVIDER_CREATE_URL}`)
  }

  const handleDeleteProvider = (providerId: string, providerName: string) => {
    deleteProvider(undefined, {
      onSuccess: () => {
        toast.success(`Provider "${providerName}" deleted`)
      },
    })
  }

  const onRowDelete = (provider: IdentityProvider) => {
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

  useEffect(() => {
    if (responseDeleteProvider) {
      toast.success('Provider deleted successfully')
    }
  }, [responseDeleteProvider])

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
