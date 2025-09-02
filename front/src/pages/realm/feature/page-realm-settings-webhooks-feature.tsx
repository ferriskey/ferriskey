import { useDeleteWebhook, useGetWebhooks } from '@/api/webhook.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import PageRealmSettingsWebhooks from '../ui/page-realm-settings-webhooks'
import { useEffect } from 'react'
import { toast } from 'sonner'

export default function PageRealmSettingsWebhooksFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data: responseGetWebhooks } = useGetWebhooks({ realm: realm_name })
  const { mutate: deleteWebhook, data: responseDeleteWebhook, isSuccess } = useDeleteWebhook()

  const handleDeleteWebhook = (webhookId: string) => {
    if (!realm_name) return
    deleteWebhook({
      path: {
        realm_name,
        webhook_id: webhookId
      }
    })
  }

  useEffect(() => {
    if (responseDeleteWebhook && isSuccess) {
      toast.success('Webhook deleted successfully')
    }
  }, [responseDeleteWebhook, isSuccess])

  return (
    <PageRealmSettingsWebhooks
      webhooks={responseGetWebhooks?.data ?? []}
      handleDeleteWebhook={handleDeleteWebhook}
    />
  )
}
