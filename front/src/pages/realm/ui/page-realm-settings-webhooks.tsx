import { DataTable } from '@/components/ui/data-table'
import { columns } from '../columns/list-webhooks.column'
import { useNavigate } from 'react-router'
import { Schemas } from '@/api/api.client'
import { Trash2 } from 'lucide-react'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'

import Webhook = Schemas.Webhook
export interface PageRealmSettingsWebhooksProps {
  webhooks: Webhook[]
  handleDeleteWebhook: (webhookId: string) => void
}

export default function PageRealmSettingsWebhooks({
  webhooks,
  handleDeleteWebhook,
}: PageRealmSettingsWebhooksProps) {
  const navigate = useNavigate()
  const { confirm, ask, close } = useConfirmDeleteAlert()
  function onRowDelete(webhook: Webhook) {
    ask({
      title: 'Delete Webhook',
      description: `Are you sure you want to delete this ${webhook.name}?`,
      onConfirm: () => {
        handleDeleteWebhook(webhook.id)
        close()
      },
    })
  }
  return (
    <>
      <DataTable
        data={webhooks}
        columns={columns}
        searchPlaceholder='Find a webhook...'
        searchKeys={['endpoint']}
        createData={{
          label: 'Create Webhook',
          onClick: () => {
            navigate('create')
          },
        }}
        rowActions={[
          {
            label: 'Delete',
            icon: <Trash2 className='h-4 w-4' />,
            variant: 'destructive',
            onClick: onRowDelete,
          },
        ]}
      />
      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </>
  )
}
