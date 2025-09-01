import { useGetWebhooks } from '@/api/webhook.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import PageRealmSettingsWebhooks from '../ui/page-realm-settings-webhooks'

export default function PageRealmSettingsWebhooksFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data: webhooks } = useGetWebhooks({ realm: realm_name })
  return <PageRealmSettingsWebhooks />
}
