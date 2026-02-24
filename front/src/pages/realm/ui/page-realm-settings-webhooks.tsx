import { useNavigate } from 'react-router'
import { Schemas } from '@/api/api.client'
import { Trash2 } from 'lucide-react'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { OverviewList } from '@/components/ui/overview-list'
import { EntityAvatar } from '@/components/ui/entity-avatar'
import { Button } from '@/components/ui/button'

import WebhookType = Schemas.Webhook

export interface PageRealmSettingsWebhooksProps {
  webhooks: WebhookType[]
  handleDeleteWebhook: (webhookId: string) => void
}

export default function PageRealmSettingsWebhooks({
  webhooks,
  handleDeleteWebhook,
}: PageRealmSettingsWebhooksProps) {
  const navigate = useNavigate()
  const { confirm, ask, close } = useConfirmDeleteAlert()

  function onRowDelete(webhook: WebhookType) {
    ask({
      title: 'Delete Webhook',
      description: `Are you sure you want to delete "${webhook.name}"? This action cannot be undone.`,
      onConfirm: () => {
        handleDeleteWebhook(webhook.id)
        close()
      },
    })
  }

  return (
    <div className='flex flex-col gap-6'>
      <OverviewList
        data={webhooks}
        searchKeys={['name', 'endpoint']}
        searchPlaceholder='Search webhooks...'
        title={(n) => `Webhooks (${n})`}
        emptyLabel='No webhooks configured.'
        action={{ label: 'New Webhook', onClick: () => navigate('create') }}
        renderRow={(webhook) => (
          <div className='flex items-center justify-between px-8 py-4 hover:bg-muted/40 transition-colors'>
            <div className='flex items-center gap-4'>
              <EntityAvatar label={webhook.name} color='#8B5CF6' />
              <div>
                <div className='flex items-center gap-2'>
                  <span className='text-base font-medium'>{webhook.name}</span>
                  <span
                    className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-mono border ${
                      webhook.enabled
                        ? 'border-green-300 text-green-600 bg-green-50 dark:bg-green-500/10 dark:border-green-400/40'
                        : 'border-border text-muted-foreground bg-muted/50'
                    }`}
                  >
                    {webhook.enabled ? 'enabled' : 'disabled'}
                  </span>
                </div>
                <div className='text-sm text-muted-foreground mt-0.5 font-mono'>{webhook.endpoint}</div>
              </div>
            </div>
            <Button
              variant='ghost'
              size='icon'
              className='text-muted-foreground hover:text-destructive'
              onClick={() => onRowDelete(webhook)}
            >
              <Trash2 className='h-4 w-4' />
            </Button>
          </div>
        )}
      />

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </div>
  )
}
