import { DataTable } from '@/components/ui/data-table'
import { columns } from '../columns/list-webhooks.column'
import { useNavigate } from 'react-router'
import { Schemas } from '@/api/api.client'
import Webhook = Schemas.Webhook
import { Trash2 } from 'lucide-react'

export interface PageRealmSettingsWebhooksProps {
  webhooks: Webhook[]
  handleDeleteWebhook: (webhookId: string) => void
}

export default function PageRealmSettingsWebhooks({ webhooks, handleDeleteWebhook }: PageRealmSettingsWebhooksProps) {
  const navigate = useNavigate()
  return (
    <div>
      <DataTable
        data={webhooks}
        columns={columns}
        searchPlaceholder='Find a webhook...'
        searchKeys={['endpoint']}
        createData={{
          label: 'Create Webhook',
          onClick: () => {
            navigate('create')
          }
        }}
        rowActions={[
          {
            label: 'Delete',
            icon: <Trash2 className='h-4 w-4' />,
            variant: 'destructive',
            onClick: (webhook) => handleDeleteWebhook(webhook.id)
          }
        ]}
      />
    </div>
  )
}
