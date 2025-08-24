import { ColumnDef } from '@/components/ui/data-table'
import { Schemas } from '@/api/api.client'
import Webhook = Schemas.Webhook

export const columns: ColumnDef<Webhook>[] = [
  {
    id: 'url',
    header: 'URL',
    cell: (webhook) => <div>{webhook.endpoint}</div>
  },
  {
    id: 'events',
    header: 'Events',
  },
  {
    id: 'status',
    header: 'Status',
  },
  {
    id: 'lastTriggeredAt',
    header: 'Last Triggered At',
  }
]
