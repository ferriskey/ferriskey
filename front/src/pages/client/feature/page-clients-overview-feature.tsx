import { Client } from '@/api/api.interface'
import { useDeleteClient, useGetClients } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { useNavigate, useParams } from 'react-router'
import PageClientsOverview from '../ui/page-clients-overview'
import {
  CLIENT_CREATE_URL,
  CLIENT_OVERVIEW_URL,
  CLIENTS_URL,
} from '@/routes/sub-router/client.router'
import { useEffect } from 'react'
import { toast } from 'sonner'

export default function PageClientsOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data, isLoading } = useGetClients({ realm: realm_name ?? 'master' })
  const { mutate: deleteClient, data: responseDeleteClient } = useDeleteClient()

  const handleDeleteSelected = (items: Client[]) => {
    if (!realm_name) return

    items.forEach((item) => {
      deleteClient({
        realm: realm_name,
        clientId: item.id,
      })
    })
  }

  const handleCreateClient = () => {
    navigate(`${CLIENTS_URL(realm_name)}${CLIENT_CREATE_URL}`)
  }

  const handleDeleteClient = (clientId: string) => {
    if (!realm_name) return

    deleteClient({
      realm: realm_name,
      clientId,
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
      data={data?.data || []}
      isLoading={isLoading}
      realmName={realm_name ?? 'master'}
      handleDeleteSelected={handleDeleteSelected}
      handleClickRow={handleClickRow}
      handleDeleteClient={handleDeleteClient}
      handleCreateClient={handleCreateClient}
    />
  )
}
