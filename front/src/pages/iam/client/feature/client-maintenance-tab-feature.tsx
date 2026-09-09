import { useState } from 'react'
import {
  useAddClientWhitelistEntry,
  useGetClientWhitelist,
  useGetRealmWhitelist,
  useRemoveClientWhitelistEntry,
  useToggleMaintenance,
} from '@/api/maintenance.api'
import { useGetUsers } from '@/api/user.api'
import { useGetRoles } from '@/api/role.api'
import { Schemas } from '@/api/api.client'
import ClientMaintenanceTab from '../ui/client-maintenance-tab'

import Client = Schemas.Client
import MaintenanceSessionStrategy = Schemas.MaintenanceSessionStrategy

export interface ClientMaintenanceTabFeatureProps {
  client: Client
  realm: string
}

export default function ClientMaintenanceTabFeature({
  client,
  realm,
}: ClientMaintenanceTabFeatureProps) {
  const { data: whitelistResponse } = useGetClientWhitelist({ realm, clientId: client.id })
  const { data: realmWhitelistResponse } = useGetRealmWhitelist({ realm })
  const { data: usersResponse } = useGetUsers({ realm })
  const { data: rolesResponse } = useGetRoles({ realm })

  const { mutate: toggleMaintenance } = useToggleMaintenance()
  const { mutate: addEntry } = useAddClientWhitelistEntry()
  const { mutate: removeEntry } = useRemoveClientWhitelistEntry()

  const [reasonOverride, setReasonOverride] = useState<string>()
  const [strategyOverride, setStrategyOverride] = useState<MaintenanceSessionStrategy>()

  const whitelist = whitelistResponse?.data ?? []
  const realmWhitelist = realmWhitelistResponse?.data ?? []
  const users = usersResponse?.data ?? []
  const roles = rolesResponse?.data ?? []

  const serverReason = client.maintenance_reason ?? ''
  const serverStrategy = client.maintenance_session_strategy ?? 'expire'
  const reason = reasonOverride ?? serverReason
  const strategy = strategyOverride ?? serverStrategy

  const userEntries = whitelist.filter((e) => e.user_id)
  const roleEntries = whitelist.filter((e) => e.role_id)

  const whitelistedUserIds = userEntries.map((e) => e.user_id as string)
  const whitelistedRoleIds = roleEntries.map((e) => e.role_id as string)

  const inheritedUsers = realmWhitelist
    .filter((e) => e.user_id)
    .map((e) => {
      const user = users.find((u) => u.id === e.user_id)
      return {
        id: e.user_id as string,
        label: user?.username ?? (e.user_id as string),
        sublabel: user?.email ?? undefined,
      }
    })

  const inheritedRoles = realmWhitelist
    .filter((e) => e.role_id)
    .map((e) => {
      const role = roles.find((r) => r.id === e.role_id)
      return {
        id: e.role_id as string,
        label: role?.name ?? (e.role_id as string),
        sublabel: role?.description ?? undefined,
      }
    })

  const hasSettingsChanges = reasonOverride !== undefined || strategyOverride !== undefined

  const sendMaintenance = (enabled: boolean) =>
    toggleMaintenance({
      body: { enabled, reason: reason || undefined, session_strategy: strategy },
      path: { client_id: client.id, realm_name: realm },
    })

  const handleWhitelistChange = (
    next: string[],
    selected: string[],
    entryIdOf: (id: string) => string | undefined,
    add: (id: string) => void
  ) => {
    const added = next.find((id) => !selected.includes(id))
    if (added !== undefined) {
      add(added)
      return
    }

    const removed = selected.find((id) => !next.includes(id))
    const entryId = removed !== undefined ? entryIdOf(removed) : undefined
    if (entryId) {
      removeEntry({ path: { client_id: client.id, realm_name: realm, entry_id: entryId } })
    }
  }

  return (
    <ClientMaintenanceTab
      enabled={client.maintenance_enabled ?? false}
      reason={reason}
      strategy={strategy}
      whitelistCount={whitelist.length}
      hasSettingsChanges={hasSettingsChanges}
      users={users.map((u) => ({ id: u.id, label: u.username, sublabel: u.email ?? undefined }))}
      roles={roles.map((r) => ({ id: r.id, label: r.name, sublabel: r.description ?? undefined }))}
      whitelistedUserIds={whitelistedUserIds}
      whitelistedRoleIds={whitelistedRoleIds}
      inheritedUsers={inheritedUsers}
      inheritedRoles={inheritedRoles}
      onToggle={sendMaintenance}
      onReasonChange={setReasonOverride}
      onStrategyChange={setStrategyOverride}
      onWhitelistedUsersChange={(next) =>
        handleWhitelistChange(
          next,
          whitelistedUserIds,
          (id) => userEntries.find((e) => e.user_id === id)?.id,
          (userId) =>
            addEntry({
              body: { user_id: userId, role_id: undefined },
              path: { client_id: client.id, realm_name: realm },
            })
        )
      }
      onWhitelistedRolesChange={(next) =>
        handleWhitelistChange(
          next,
          whitelistedRoleIds,
          (id) => roleEntries.find((e) => e.role_id === id)?.id,
          (roleId) =>
            addEntry({
              body: { user_id: undefined, role_id: roleId },
              path: { client_id: client.id, realm_name: realm },
            })
        )
      }
      onSaveSettings={() => sendMaintenance(client.maintenance_enabled ?? false)}
      onResetSettings={() => {
        setReasonOverride(undefined)
        setStrategyOverride(undefined)
      }}
    />
  )
}
