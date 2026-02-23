import { Schemas } from '@/api/api.client.ts'
import BadgeColor from '@/components/ui/badge-color'

import { ColumnDef } from '@/components/ui/data-table'

import Client = Schemas.Client
import { BadgeColorScheme } from '@/components/ui/badge-color.enum'
import { AlertTriangle } from 'lucide-react'

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
        {client.public_client ? 'public' : 'confidential'}
      </BadgeColor>
    ),
  },
  {
    id: 'status',
    header: 'Status',
    cell: (client) => {
      if (!client.enabled) {
        return (
          <span className='inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-semibold border border-orange-500/40 text-orange-500 bg-orange-500/10'>
            <AlertTriangle className='h-3 w-3' />
            RISK
          </span>
        )
      }
      return (
        <span className='inline-flex items-center px-2.5 py-1 rounded-md text-xs font-semibold border border-emerald-500/40 text-emerald-600 bg-emerald-500/10'>
          ACTIVE
        </span>
      )
    },
  },
]
