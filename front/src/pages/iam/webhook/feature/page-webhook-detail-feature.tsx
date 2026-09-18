import { useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useTranslation } from 'react-i18next'
import { useDeleteWebhook, useGetWebhook, useUpdateWebhook } from '@/api/webhook.api'
import { RouterParams } from '@/routes/router'
import { useRouteTabs } from '@/components/kit'
import { updateWebhookValidator } from '@/pages/iam/realm/validators'
import { Schemas } from '@/api/api.client'
import { WEBHOOKS_URL } from '@/routes/router'
import PageWebhookDetail from '../ui/page-webhook-detail'
import type { WebhookHeader } from '../ui/webhook-headers-field'
import { apiErrorMessage, partitionFieldErrors, validationErrorsFrom } from '@/lib/api-error'

import WebhookTrigger = Schemas.WebhookTrigger
import { useCrumbLabel } from '@/components/shell/crumb-store'

type WebhookField = 'name' | 'endpoint' | 'description'

const FORM_FIELDS: WebhookField[] = ['name', 'endpoint', 'description']

export interface RetryPolicyDraft {
  max_attempts: string
  base_delay_ms: string
  max_delay_ms: string
  max_total_delay_ms: string
}

interface Draft {
  key: string
  name: string
  endpoint: string
  description: string
  headers: WebhookHeader[]
  subscribers: WebhookTrigger[]
  retryPolicy: RetryPolicyDraft
}

const EMPTY_RETRY_POLICY: RetryPolicyDraft = {
  max_attempts: '',
  base_delay_ms: '',
  max_delay_ms: '',
  max_total_delay_ms: '',
}

const asField = (value?: number | null) => (value === null || value === undefined ? '' : String(value))

const asNumber = (value: string) => {
  const trimmed = value.trim()
  if (trimmed === '') return null
  const parsed = Number(trimmed)
  return Number.isFinite(parsed) ? parsed : null
}

const EMPTY_DRAFT: Draft = {
  key: '',
  name: '',
  endpoint: '',
  description: '',
  headers: [],
  subscribers: [],
  retryPolicy: EMPTY_RETRY_POLICY,
}

const WEBHOOK_TABS = [
  { key: 'settings', labelKey: 'detail.tabs.settings' },
  { key: 'events', labelKey: 'detail.tabs.events' },
  { key: 'deliveries', labelKey: 'detail.tabs.deliveries' },
] as const

export default function PageWebhookDetailFeature() {
  const { realm_name, webhook_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { t } = useTranslation('webhook')
  const realm = realm_name ?? 'master'

  const { data: webhook, isLoading } = useGetWebhook({
    realm,
    webhookId: webhook_id ?? '',
  })
  const { mutate: updateWebhook } = useUpdateWebhook()
  const { mutate: deleteWebhook } = useDeleteWebhook()

  const translatedTabs = useMemo(
    () => WEBHOOK_TABS.map((item) => ({ key: item.key, label: t(item.labelKey) })),
    [t]
  )

  const { value: tab, tabs } = useRouteTabs(
    `${WEBHOOKS_URL(realm)}/${webhook_id ?? ''}`,
    translatedTabs
  )

  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT)
  const [serverErrors, setServerErrors] = useState<
    Partial<Record<WebhookField, string>>
  >({})

  const webhookKey = webhook ? `${webhook.id}:${webhook.updated_at}` : ''
  const pristine: Draft = webhook
    ? {
        key: webhookKey,
        name: webhook.name ?? '',
        endpoint: webhook.endpoint,
        description: webhook.description ?? '',
        headers: [],
        subscribers: webhook.subscribers.map((subscriber) => subscriber.name),
        retryPolicy: {
          max_attempts: asField(webhook.retry_policy?.max_attempts),
          base_delay_ms: asField(webhook.retry_policy?.base_delay_ms),
          max_delay_ms: asField(webhook.retry_policy?.max_delay_ms),
          max_total_delay_ms: asField(webhook.retry_policy?.max_total_delay_ms),
        },
      }
    : EMPTY_DRAFT

  if (webhook && draft.key !== webhookKey) setDraft(pristine)

  const current = draft.key === webhookKey ? draft : pristine

  const parsed = updateWebhookValidator.safeParse({
    name: current.name,
    endpoint: current.endpoint,
    description: current.description,
    subscribers: current.subscribers,
    headers: current.headers,
  })

  const errors: Partial<Record<WebhookField, string>> = { ...serverErrors }
  if (!parsed.success) {
    for (const issue of parsed.error.issues) {
      const field = issue.path[0] as WebhookField
      if (!errors[field]) errors[field] = issue.message
    }
  }

  const dirtyCount = webhook
    ? (current.name !== pristine.name ? 1 : 0) +
      (current.endpoint !== pristine.endpoint ? 1 : 0) +
      (current.description !== pristine.description ? 1 : 0) +
      (current.headers.length > 0 ? 1 : 0) +
      (current.subscribers.length !== pristine.subscribers.length ||
      current.subscribers.some((s) => !pristine.subscribers.includes(s))
        ? 1
        : 0) +
      (JSON.stringify(current.retryPolicy) !== JSON.stringify(pristine.retryPolicy) ? 1 : 0)
    : 0

  const patch = (next: Partial<Draft>) =>
    setDraft({ ...current, ...next, key: webhookKey })

  const save = () => {
    if (!realm_name || !webhook_id || !parsed.success) return

    const entered = current.headers.filter((header) => header.key.trim().length > 0)
    const headers = entered.length
      ? entered.reduce<Record<string, string>>((acc, header) => {
          acc[header.key] = header.value
          return acc
        }, {})
      : undefined

    updateWebhook(
      {
        body: {
          name: current.name,
          endpoint: current.endpoint,
          description: current.description,
          subscribers: current.subscribers,
          retry_policy: {
            max_attempts: asNumber(current.retryPolicy.max_attempts),
            base_delay_ms: asNumber(current.retryPolicy.base_delay_ms),
            max_delay_ms: asNumber(current.retryPolicy.max_delay_ms),
            max_total_delay_ms: asNumber(current.retryPolicy.max_total_delay_ms),
          },
          ...(headers ? { headers } : {}),
        },
        path: { realm_name, webhook_id },
      },
      {
        onSuccess: () => {
          toast.success(t('detail.toast.updated'))
          navigate(WEBHOOKS_URL(realm))
        },
        onError: (error: unknown) => {
          const fieldErrors = validationErrorsFrom(error)

          if (fieldErrors.length > 0) {
            const { byField, unattached } = partitionFieldErrors(fieldErrors, (field) =>
              FORM_FIELDS.includes(field as WebhookField) ? (field as WebhookField) : undefined
            )

            setServerErrors(Object.fromEntries(byField) as Partial<Record<WebhookField, string>>)
            if (unattached.length > 0) toast.error(unattached.join(' · '))
            return
          }

          toast.error(apiErrorMessage(error, 'Failed to update webhook'))
        },
      }
    )
  }

  const handleDelete = () => {
    if (!realm_name || !webhook_id) return

    deleteWebhook(
      { path: { realm_name, webhook_id } },
      {
        onSuccess: () => {
          toast.success(t('detail.toast.deleted'))
          navigate(WEBHOOKS_URL(realm))
        },
      }
    )
  }


  useCrumbLabel(webhook_id, webhook?.name)

  return (
    <PageWebhookDetail
      realm={realm}
      webhook={webhook}
      isLoading={isLoading}
      tab={tab}
      tabs={tabs}
      name={current.name}
      endpoint={current.endpoint}
      description={current.description}
      headers={current.headers}
      subscribers={current.subscribers}
      retryPolicy={current.retryPolicy}
      effectiveRetryPolicy={webhook?.effective_retry_policy ?? undefined}
      errors={errors}
      dirtyCount={dirtyCount}
      onNameChange={(v) => patch({ name: v })}
      onEndpointChange={(v) => patch({ endpoint: v })}
      onDescriptionChange={(v) => patch({ description: v })}
      onHeadersChange={(next) => patch({ headers: next })}
      onSubscribersChange={(next) => patch({ subscribers: next })}
      onRetryPolicyChange={(next) => patch({ retryPolicy: { ...current.retryPolicy, ...next } })}
      onBack={() => navigate(WEBHOOKS_URL(realm))}
      onDiscard={() => setDraft(pristine)}
      onSave={save}
      onDelete={handleDelete}
    />
  )
}
