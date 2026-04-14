import { useState } from 'react'
import { useParams } from 'react-router'
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
import { Button } from '@/components/ui/button'
import { Trash2 } from 'lucide-react'
import { Schemas } from '@/api/api.client'
import {
  useToggleMaintenance,
  useGetClientWhitelist,
  useAddClientWhitelistEntry,
  useRemoveClientWhitelistEntry,
} from '@/api/maintenance.api'
import { RouterParams } from '@/routes/router'
import Client = Schemas.Client

interface ClientMaintenanceSectionProps {
  client: Client
}

export default function ClientMaintenanceSection({ client }: ClientMaintenanceSectionProps) {
  const { realm_name } = useParams<RouterParams>()
  const realmName = realm_name ?? 'master'

  const [reason, setReason] = useState(client.maintenance_reason ?? '')
  const [strategy, setStrategy] = useState<'terminate' | 'expire'>(
    (client.maintenance_session_strategy as 'terminate' | 'expire') ?? 'expire'
  )
  const [newUserId, setNewUserId] = useState('')
  const [newRoleId, setNewRoleId] = useState('')

  const { mutate: toggleMaintenance } = useToggleMaintenance()
  const { data: whitelistResponse } = useGetClientWhitelist({
    realm: realmName,
    clientId: client.id,
  })
  const { mutate: addEntry } = useAddClientWhitelistEntry()
  const { mutate: removeEntry } = useRemoveClientWhitelistEntry()

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const whitelist: Array<{ id: string; user_id?: string; role_id?: string }> = (whitelistResponse as any)?.data ?? []

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

  const handleAddUser = () => {
    if (!newUserId.trim()) return
    addEntry({
      body: { user_id: newUserId.trim(), role_id: undefined },
      path: { client_id: client.id, realm_name: realmName },
    })
    setNewUserId('')
  }

  const handleAddRole = () => {
    if (!newRoleId.trim()) return
    addEntry({
      body: { user_id: undefined, role_id: newRoleId.trim() },
      path: { client_id: client.id, realm_name: realmName },
    })
    setNewRoleId('')
  }

  const handleRemove = (entryId: string) => {
    removeEntry({
      path: { client_id: client.id, realm_name: realmName, entry_id: entryId },
    })
  }

  return (
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
            onChange={(v) => setReason(String(v ?? ''))}
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
          <Select onValueChange={(v) => setStrategy(v as 'terminate' | 'expire')} value={strategy}>
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

      {/* Whitelist */}
      <div className='flex flex-col gap-4 py-4 border-t'>
        <div>
          <p className='text-sm font-medium'>Whitelist</p>
          <p className='text-sm text-muted-foreground mt-0.5'>
            Users and roles allowed to authenticate during maintenance. Realm-level entries are inherited automatically.
          </p>
        </div>

        {/* Add user */}
        <div className='flex items-end gap-2'>
          <div className='flex-1'>
            <InputText
              label='User ID'
              value={newUserId}
              name='whitelist_user_id'
              onChange={(v) => setNewUserId(String(v ?? ''))}
            />
          </div>
          <Button type='button' variant='outline' size='sm' onClick={handleAddUser}>
            Add user
          </Button>
        </div>

        {/* Add role */}
        <div className='flex items-end gap-2'>
          <div className='flex-1'>
            <InputText
              label='Role ID'
              value={newRoleId}
              name='whitelist_role_id'
              onChange={(v) => setNewRoleId(String(v ?? ''))}
            />
          </div>
          <Button type='button' variant='outline' size='sm' onClick={handleAddRole}>
            Add role
          </Button>
        </div>

        {/* Entries list */}
        {whitelist.length > 0 && (
          <div className='rounded-md border'>
            <table className='w-full text-sm'>
              <thead>
                <tr className='border-b bg-muted/50'>
                  <th className='px-4 py-2 text-left font-medium'>Type</th>
                  <th className='px-4 py-2 text-left font-medium'>ID</th>
                  <th className='px-4 py-2 text-right font-medium'>Actions</th>
                </tr>
              </thead>
              <tbody>
                {whitelist.map((entry) => (
                  <tr key={entry.id} className='border-b last:border-0'>
                    <td className='px-4 py-2 text-muted-foreground'>
                      {entry.user_id ? 'User' : 'Role'}
                    </td>
                    <td className='px-4 py-2 font-mono text-xs'>
                      {entry.user_id ?? entry.role_id}
                    </td>
                    <td className='px-4 py-2 text-right'>
                      <Button
                        type='button'
                        variant='ghost'
                        size='sm'
                        onClick={() => handleRemove(entry.id)}
                      >
                        <Trash2 className='h-4 w-4 text-destructive' />
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  )
}
