import { useNavigate, useParams } from 'react-router'
import {
  WEBHOOK_FILTER_KEYS,
  useGetWebhooks,
  useWebhookCount,
  type WebhooksQuery,
} from '@/api/webhook.api'
import { RouterParams } from '@/routes/router'
import { usePagedListing } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { WEBHOOKS_URL } from '@/routes/router'
import PageWebhooksOverview from '../ui/page-webhooks-overview'

import Webhook = Schemas.Webhook

const ALERT_PREVIEW = 5

const label = (webhook: Webhook) => webhook.name || webhook.endpoint

export default function PageWebhooksOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const listing = usePagedListing(WEBHOOK_FILTER_KEYS)
  const { data: response, isLoading } = useGetWebhooks({
    realm,
    query: listing.apiQuery as WebhooksQuery,
  })
  const total = useWebhookCount({ realm })
  const never = useWebhookCount({ realm, filter: { triggered: false } })
  const { data: silent } = useGetWebhooks({
    realm,
    query: { has_subscribers: false, limit: ALERT_PREVIEW },
  })
  const { data: insecure } = useGetWebhooks({
    realm,
    query: { secure_endpoint: false, limit: ALERT_PREVIEW },
  })

  return (
    <PageWebhooksOverview
      webhooks={response?.data ?? []}
      pagination={response?.metadata}
      listing={listing}
      isLoading={isLoading}
      counts={{
        total: total.count,
        never: never.count,
        silent: silent?.metadata.total ?? 0,
      }}
      silent={{
        total: silent?.metadata.total ?? 0,
        names: (silent?.data ?? []).map(label),
      }}
      insecure={{
        total: insecure?.metadata.total ?? 0,
        names: (insecure?.data ?? []).map(label),
      }}
      webhookHref={(webhook: Webhook) => `${WEBHOOKS_URL(realm)}/${webhook.id}/settings`}
      onCreate={() => navigate(`${WEBHOOKS_URL(realm)}/create`)}
    />
  )
}
