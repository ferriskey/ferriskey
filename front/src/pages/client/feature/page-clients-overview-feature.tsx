import { useDeleteClient, useGetClients } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { useNavigate, useParams } from 'react-router'
import PageClientsOverview from '../ui/page-clients-overview'
import {
  CLIENT_CREATE_URL,
  CLIENT_OVERVIEW_URL,
  CLIENTS_URL,
} from '@/routes/sub-router/client.router'
import { useEffect, useState, useMemo } from 'react'
import { toast } from 'sonner'
import { Schemas } from '@/api/api.client.ts'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import Client = Schemas.Client

export default function PageClientsOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data, isLoading } = useGetClients({ realm: realm_name ?? 'master' })
  const { mutate: deleteClient, data: responseDeleteClient } = useDeleteClient()
  const { confirm, ask, close } = useConfirmDeleteAlert()
  const [filters, setFilters] = useState<Filter[]>([])

  const clients = useMemo(() => data?.data || [], [data])

  const filterFields: FilterFieldsConfig = [
    {
      key: 'name',
      label: 'Client Name',
      type: 'text',
    },
    {
      key: 'client_id',
      label: 'Client ID',
      type: 'text',
    },
    {
      key: 'public_client',
      label: 'Type',
      type: 'boolean',
    },
    {
      key: 'enabled',
      label: 'Status',
      type: 'boolean',
    },
  ]

  const filteredData = useMemo(() => {
    if (filters.length === 0) return clients

    return clients.filter((client) => {
      return filters.every((filter) => {
        const fieldValue = client[filter.field as keyof Client]
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
  }, [clients, filters])

  const statistics = useMemo(() => {
    return {
      totalClients: clients.length,
      activeClients: clients.filter((client) => client.enabled).length,
      publicClients: clients.filter((client) => client.public_client).length,
      confidentialClients: clients.filter((client) => !client.public_client).length,
    }
  }, [clients])

  const handleDeleteSelected = (items: Client[]) => {
    if (!realm_name) return

    items.forEach((item) => {
      deleteClient({
        path: {
          client_id: item.id,
          realm_name
        }
      })
    })
  }

  const handleCreateClient = () => {
    navigate(`${CLIENTS_URL(realm_name)}${CLIENT_CREATE_URL}`)
  }

  const handleDeleteClient = (clientId: string) => {
    if (!realm_name) return

    deleteClient({
      path: {
        client_id: clientId,
        realm_name
      }
    })
  }

  const onRowDelete = (client: Client) => {
    ask({
      title: 'Delete client?',
      description: `Are you sure you want to delete "${client.name}"?`,
      onConfirm: () => {
        handleDeleteClient(client.id)
        close()
      },
    })
  }

  const handleClickRow = (clientId: string) => {
    navigate(`${CLIENT_OVERVIEW_URL(realm_name, clientId)}`)
  }

  useEffect(() => {
    if (responseDeleteClient) {
      toast.success('Client deleted successfully')
    }
  }, [responseDeleteClient])

  return (
    <PageClientsOverview
      data={filteredData}
      isLoading={isLoading}
      realmName={realm_name ?? 'master'}
      statistics={statistics}
      filters={filters}
      filterFields={filterFields}
      onFiltersChange={setFilters}
      confirm={confirm}
      onConfirmClose={close}
      handleDeleteSelected={handleDeleteSelected}
      handleClickRow={handleClickRow}
      handleCreateClient={handleCreateClient}
      onRowDelete={onRowDelete}
    />
  )
}
