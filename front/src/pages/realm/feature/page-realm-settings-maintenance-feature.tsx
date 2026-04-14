import { useParams } from 'react-router'
import { useGetUsers } from '@/api/user.api'
import { useGetRoles } from '@/api/role.api'
import {
  useGetRealmWhitelist,
  useAddRealmWhitelistEntry,
  useRemoveRealmWhitelistEntry,
} from '@/api/maintenance.api'
import { RouterParams } from '@/routes/router'
import WhitelistPicker from '@/pages/client/components/whitelist-picker'

export default function PageRealmSettingsMaintenanceFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realmName = realm_name ?? 'master'

  const { data: usersResponse } = useGetUsers({ realm: realmName })
  const { data: rolesResponse } = useGetRoles({ realm: realmName })
  const { data: whitelistResponse } = useGetRealmWhitelist({ realm: realmName })
  const { mutate: addEntry } = useAddRealmWhitelistEntry()
  const { mutate: removeEntry } = useRemoveRealmWhitelistEntry()


  const whitelist: Array<{ id: string; user_id?: string; role_id?: string }> =
    (whitelistResponse as unknown as { data?: Array<{ id: string; user_id?: string; role_id?: string }> })?.data ?? []

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const users = ((usersResponse as any)?.data ?? []) as Array<{
    id: string
    username: string
    email: string
  }>
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const roles = ((rolesResponse as any)?.data ?? []) as Array<{
    id: string
    name: string
    description?: string
  }>

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

  const handleAddUser = (userId: string) => {
    addEntry({
      body: { user_id: userId, role_id: undefined },
      path: { realm_name: realmName },
    })
  }

  const handleAddRole = (roleId: string) => {
    addEntry({
      body: { user_id: undefined, role_id: roleId },
      path: { realm_name: realmName },
    })
  }

  const handleRemoveEntry = (entryId: string) => {
    removeEntry({
      path: { realm_name: realmName, entry_id: entryId },
    })
  }

  return (
    <div className='flex flex-col gap-1'>
      <div className='mb-4'>
        <p className='text-xs text-muted-foreground mb-0.5'>Realm-wide maintenance configuration</p>
        <h2 className='text-base font-semibold'>Default Maintenance Members</h2>
        <p className='text-sm text-muted-foreground mt-1'>
          Users and roles listed here are automatically allowed to authenticate on any client under
          maintenance in this realm.
        </p>
      </div>

      <WhitelistPicker
        title='Users'
        description='Individual users always allowed during maintenance.'
        items={users.map((u) => ({
          id: u.id,
          label: u.username,
          sublabel: u.email,
        }))}
        whitelistedIds={whitelistedUserIds}
        onAdd={handleAddUser}
        onRemove={handleRemoveEntry}
        entryIdMap={userEntryIdMap}
        placeholder='Search users...'
        emptyMessage='No users available.'
      />

      <WhitelistPicker
        title='Roles'
        description='All users with these roles are always allowed during maintenance.'
        items={roles.map((r) => ({
          id: r.id,
          label: r.name,
          sublabel: r.description,
        }))}
        whitelistedIds={whitelistedRoleIds}
        onAdd={handleAddRole}
        onRemove={handleRemoveEntry}
        entryIdMap={roleEntryIdMap}
        placeholder='Search roles...'
        emptyMessage='No roles available.'
      />
    </div>
  )
}
