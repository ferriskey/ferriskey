import { ColumnDef } from '@/components/ui/data-table'
import { Schemas } from '@/api/api.client'
import Webhook = Schemas.Webhook
import BadgeColor from '@/components/ui/badge-color'
import { BadgeColorScheme } from '@/components/ui/badge-color.enum'

export const columns: ColumnDef<Webhook>[] = [
  {
    id: 'url',
    header: 'URL',
    cell: (webhook) => <div><BadgeColor color={BadgeColorScheme.PRIMARY}>{webhook.endpoint}</BadgeColor></div>
  },
  {
    id: 'name',
    header: 'Name',
    cell: (webhook) => <div>{webhook.name}</div>
  },
  {
    id: 'status',
    header: 'Status',
    cell: () => <div><BadgeColor color={BadgeColorScheme.GREEN}>Active</BadgeColor></div>
  },
  {
    id: 'lastTriggeredAt',
    header: 'Last Triggered At',
    cell: (webhook) => (
      <div>
        <BadgeColor color={BadgeColorScheme.GRAY}>{webhook.triggered_at ? new Date(webhook.triggered_at).toLocaleString() : 'Never'}</BadgeColor>
      </div>
    )
  }
]
