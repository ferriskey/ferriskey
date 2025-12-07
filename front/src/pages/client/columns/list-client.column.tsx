import { Schemas } from '@/api/api.client.ts'
import BadgeColor from '@/components/ui/badge-color'

import { ColumnDef } from '@/components/ui/data-table'

import Client = Schemas.Client
import { BadgeColorScheme } from '@/components/ui/badge-color.enum'

export const columns: ColumnDef<Client>[] = [
  {
    id: 'name',
    header: 'Client',
    cell: (client) => (
      <div className='flex items-center gap-3'>
        <div className='h-10 w-10 rounded-lg bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center border border-primary/20'>
          <span className='text-sm font-semibold text-primary'>{client.name?.[0]?.toUpperCase() || 'C'}</span>
        </div>
        <div className='flex flex-col'>
          <div className='font-semibold text-sm'>{client.name}</div>
          <div className='text-xs text-muted-foreground font-mono'>{client.client_id}</div>
        </div>
      </div>
    ),
  },
  {
    id: 'type',
    header: 'Type',
    cell: (client) => (
      <BadgeColor color={client.public_client ? BadgeColorScheme.BLUE : BadgeColorScheme.PURPLE}>
        {client.public_client ? 'Public' : 'Confidential'}
      </BadgeColor>
    ),
  },
  {
    id: 'status',
    header: 'Status',
    cell: (client) => (
      <div className='flex items-center gap-2'>
        <span
          className={`h-2 w-2 rounded-full ${client.enabled ? 'bg-emerald-500' : 'bg-red-500'}`}
        ></span>
        <span className='text-sm'>{client.enabled ? 'Active' : 'Inactive'}</span>
      </div>
    ),
  },
]
