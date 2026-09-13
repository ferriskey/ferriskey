import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useGetWebhooks } from '@/api/webhook.api'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { WEBHOOKS_URL } from '@/routes/router'
import PageWebhooksOverview from '../ui/page-webhooks-overview'

import Webhook = Schemas.Webhook

export default function PageWebhooksOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: webhooksResponse, isLoading } = useGetWebhooks({ realm })
  const webhooks = useMemo(() => webhooksResponse?.data ?? [], [webhooksResponse])

  return (
    <PageWebhooksOverview
      webhooks={webhooks}
      isLoading={isLoading}
      webhookHref={(webhook: Webhook) =>
        `${WEBHOOKS_URL(realm)}/${webhook.id}/settings`
      }
      onCreate={() => navigate(`${WEBHOOKS_URL(realm)}/create`)}
    />
  )
}
