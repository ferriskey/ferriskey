import { useMemo } from 'react'
import { useGetClients } from '@/api/client.api'
import { useGetRoles } from '@/api/role.api'
import type { MapperEntityOptions } from '../ui/mapper-config-fields'

export function useMapperEntityOptions(realm: string): MapperEntityOptions {
  const { data: clientsResponse } = useGetClients({ realm })
  const { data: rolesResponse } = useGetRoles({ realm })

  return useMemo(() => {
    const clients = (clientsResponse?.data ?? []).map((client) => ({
      value: client.client_id,
      label: client.name,
      sublabel: client.client_id,
    }))

    const roles = (rolesResponse?.data ?? []).map((role) => {
      const clientId = role.client?.client_id
      return {
        value: clientId ? `${clientId}.${role.name}` : role.name,
        label: role.name,
        sublabel: clientId ? `${clientId}.${role.name}` : 'realm role',
      }
    })

    return { client: clients, role: roles }
  }, [clientsResponse, rolesResponse])
}
