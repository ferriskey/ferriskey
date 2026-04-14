import { useGetClient } from '@/api/client.api'
import { useGetUsers } from '@/api/user.api'
import { useGetRoles } from '@/api/role.api'
import {
  useToggleMaintenance,
  useGetClientWhitelist,
  useAddClientWhitelistEntry,
  useRemoveClientWhitelistEntry,
  useGetRealmWhitelist,
} from '@/api/maintenance.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import { useState } from 'react'
import { Switch } from '@/components/ui/switch'
import { InputText } from '@/components/ui/input-text'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import WhitelistPicker from '../components/whitelist-picker'

export default function PageClientMaintenanceFeature() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const realmName = realm_name ?? 'master'

  const { data: clientResponse } = useGetClient({
    realm: realmName,
    clientId: client_id,
  })

  const { data: usersResponse } = useGetUsers({ realm: realmName })
  const { data: rolesResponse } = useGetRoles({ realm: realmName })


  const { data: whitelistResponse } = useGetClientWhitelist({
    realm: realmName,
    clientId: client_id,
  })

  const { data: realmWhitelistResponse } = useGetRealmWhitelist({ realm: realmName })

  const { mutate: toggleMaintenance } = useToggleMaintenance()
  const { mutate: addEntry } = useAddClientWhitelistEntry()
  const { mutate: removeEntry } = useRemoveClientWhitelistEntry()

  const client = clientResponse?.data


  const whitelist = whitelistResponse?.data ?? []
  const realmWhitelist = realmWhitelistResponse?.data ?? []

  // Track user edits — undefined means "not edited, use server value"
  const [reasonOverride, setReasonOverride] = useState<string | undefined>(undefined)
  const [strategyOverride, setStrategyOverride] = useState<'terminate' | 'expire' | undefined>(
    undefined
  )

  if (!client) return null

  const serverReason = client.maintenance_reason ?? ''
  const serverStrategy =
    (client.maintenance_session_strategy as 'terminate' | 'expire') ?? 'expire'
  const reason = reasonOverride ?? serverReason
  const strategy = strategyOverride ?? serverStrategy

  const users = usersResponse?.data ?? []
  const roles = rolesResponse?.data ?? []

  const whitelistedUserIds = whitelist.filter((e) => e.user_id).map((e) => e.user_id!)
  const whitelistedRoleIds = whitelist.filter((e) => e.role_id).map((e) => e.role_id!)

  const userEntryIdMap: Record<string, string> = {}
  whitelist
    .filter((e) => e.user_id)
    .forEach((e) => {
      userEntryIdMap[e.user_id!] = e.id
    })

  const roleEntryIdMap: Record<string, string> = {}
  whitelist
    .filter((e) => e.role_id)
    .forEach((e) => {
      roleEntryIdMap[e.role_id!] = e.id
    })

  const inheritedUsers = realmWhitelist
    .filter((e) => e.user_id)
    .map((e) => {
      const user = users.find((u) => u.id === e.user_id)
      return { id: e.user_id!, label: user?.username ?? e.user_id!, sublabel: user?.email }
    })

  const inheritedRoles = realmWhitelist
    .filter((e) => e.role_id)
    .map((e) => {
      const role = roles.find((r) => r.id === e.role_id)
      return { id: e.role_id!, label: role?.name ?? e.role_id!, sublabel: role?.description }
    })

  const handleToggle = (enabled: boolean) => {
    toggleMaintenance({
      body: {
        enabled,
        reason: reason || undefined,
        session_strategy: strategy,
      },
      path: {
        client_id: client.id,
        realm_name: realmName,
      },
    })
  }

  const hasSettingsChanges = reasonOverride !== undefined || strategyOverride !== undefined

  const handleSaveSettings = () => {
    toggleMaintenance({
      body: {
        enabled: client.maintenance_enabled ?? false,
        reason: reason || undefined,
        session_strategy: strategy,
      },
      path: {
        client_id: client.id,
        realm_name: realmName,
      },
    })
  }

  const handleResetSettings = () => {
    setReasonOverride(undefined)
    setStrategyOverride(undefined)
  }

  const handleAddUser = (userId: string) => {
    addEntry({
      body: { user_id: userId, role_id: undefined },
      path: { client_id: client.id, realm_name: realmName },
    })
  }

  const handleAddRole = (roleId: string) => {
    addEntry({
      body: { user_id: undefined, role_id: roleId },
      path: { client_id: client.id, realm_name: realmName },
    })
  }

  const handleRemoveEntry = (entryId: string) => {
    removeEntry({
      path: { client_id: client.id, realm_name: realmName, entry_id: entryId },
    })
  }

  return (
    <div className='flex flex-col gap-8'>
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Restrict access temporarily</p>
          <h2 className='text-base font-semibold'>Maintenance Mode</h2>
        </div>

        {/* Toggle */}
        <div className='flex items-center justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Maintenance Enabled</p>
            <p className='text-sm text-muted-foreground mt-0.5'>
              When enabled, only whitelisted users and roles can authenticate.
            </p>
          </div>
          <div className='w-1/2'>
            <div className='flex flex-row items-center gap-3'>
              <Switch
                checked={client.maintenance_enabled ?? false}
                onCheckedChange={handleToggle}
              />
              <span className='text-sm font-normal text-muted-foreground'>
                {client.maintenance_enabled ? 'Enabled' : 'Disabled'}
              </span>
            </div>
          </div>
        </div>

        {/* Reason */}
        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Reason</p>
            <p className='text-sm text-muted-foreground mt-0.5'>
              Message displayed to blocked users on the login page.
            </p>
          </div>
          <div className='w-1/2'>
            <InputText
              label='Maintenance reason'
              value={reason}
              name='maintenance_reason'
              onChange={(v) => setReasonOverride(String(v ?? ''))}
            />
          </div>
        </div>

        {/* Session Strategy */}
        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Session Strategy</p>
            <p className='text-sm text-muted-foreground mt-0.5'>
              How existing sessions are handled when maintenance is enabled.
            </p>
          </div>
          <div className='w-1/2'>
            <Label className='text-sm text-muted-foreground mb-1.5 block'>Strategy</Label>
            <Select
              onValueChange={(v) => setStrategyOverride(v as 'terminate' | 'expire')}
              value={strategy}
            >
              <SelectTrigger className='w-48'>
                <SelectValue>{strategy === 'terminate' ? 'Terminate' : 'Expire'}</SelectValue>
              </SelectTrigger>
              <SelectContent position='popper'>
                <SelectItem value='expire'>Expire naturally</SelectItem>
                <SelectItem value='terminate'>Terminate immediately</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </div>

      {/* Whitelist */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Access control</p>
          <h2 className='text-base font-semibold'>Whitelist</h2>
          <p className='text-sm text-muted-foreground mt-1'>
            Users and roles allowed to authenticate during maintenance. Realm-level entries are
            inherited automatically.
          </p>
        </div>

        <WhitelistPicker
          title='Users'
          description='Individual users allowed during maintenance.'
          items={users.map((u) => ({
            id: u.id,
            label: u.username,
            sublabel: u.email,
          }))}
          whitelistedIds={whitelistedUserIds}
          onAdd={handleAddUser}
          onRemove={handleRemoveEntry}
          entryIdMap={userEntryIdMap}
          inheritedEntries={inheritedUsers}
          placeholder='Search users...'
          emptyMessage='No users available.'
        />

        <WhitelistPicker
          title='Roles'
          description='All users with these roles are allowed during maintenance.'
          items={roles.map((r) => ({
            id: r.id,
            label: r.name,
            sublabel: r.description,
          }))}
          whitelistedIds={whitelistedRoleIds}
          onAdd={handleAddRole}
          onRemove={handleRemoveEntry}
          entryIdMap={roleEntryIdMap}
          inheritedEntries={inheritedRoles}
          placeholder='Search roles...'
          emptyMessage='No roles available.'
        />
      </div>

      <FloatingActionBar
        show={hasSettingsChanges}
        title='Save Changes'
        description='Save maintenance reason and session strategy changes.'
        actions={[
          {
            label: 'Save',
            variant: 'default',
            onClick: handleSaveSettings,
          },
        ]}
        onCancel={handleResetSettings}
      />
    </div>
  )
}
