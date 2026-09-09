import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useDeleteWebhook, useGetWebhook, useUpdateWebhook } from '@/api/webhook.api'
import { RouterParams } from '@/routes/router'
import { useRouteTabs } from '@/components/kit'
import { updateWebhookValidator } from '@/pages/realm/validators'
import { Schemas } from '@/api/api.client'
import { NEXT_WEBHOOKS_URL } from '@/next/routes'
import PageWebhookDetail from '../ui/page-webhook-detail'
import type { WebhookHeader } from '../ui/webhook-headers-field'

import WebhookTrigger = Schemas.WebhookTrigger
import { useCrumbLabel } from '@/next/shell/crumb-store'

type WebhookField = 'name' | 'endpoint' | 'description'

interface Draft {
  key: string
  name: string
  endpoint: string
  description: string
  headers: WebhookHeader[]
  subscribers: WebhookTrigger[]
}

const EMPTY_DRAFT: Draft = {
  key: '',
  name: '',
  endpoint: '',
  description: '',
  headers: [],
  subscribers: [],
}

const WEBHOOK_TABS = [
  { key: 'settings', label: 'Settings' },
  { key: 'events', label: 'Events' },
  { key: 'deliveries', label: 'Deliveries' },
] as const

export default function PageWebhookDetailFeature() {
  const { realm_name, webhook_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: webhook, isLoading } = useGetWebhook({
    realm,
    webhookId: webhook_id ?? '',
  })
  const { mutate: updateWebhook } = useUpdateWebhook()
  const { mutate: deleteWebhook } = useDeleteWebhook()

  const { value: tab, tabs } = useRouteTabs(
    `${NEXT_WEBHOOKS_URL(realm)}/${webhook_id ?? ''}`,
    WEBHOOK_TABS
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
        : 0)
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
          ...(headers ? { headers } : {}),
        },
        path: { realm_name, webhook_id },
      },
      {
        onSuccess: () => {
          toast.success('Webhook updated successfully')
          navigate(NEXT_WEBHOOKS_URL(realm))
        },
        onError: (error: unknown) => {
          const body = error as {
            message?: string
            data?: { errors?: { field: string; message: string }[] }
          }
          const fieldErrors = body?.data?.errors

          if (Array.isArray(fieldErrors) && fieldErrors.length > 0) {
            const next: Partial<Record<WebhookField, string>> = {}
            for (const { field, message } of fieldErrors) {
              next[field as WebhookField] = message
            }
            setServerErrors(next)
          } else {
            toast.error(body?.message ?? 'Failed to update webhook')
          }
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
          toast.success('Webhook deleted successfully')
          navigate(NEXT_WEBHOOKS_URL(realm))
        },
      }
    )
  }


  useCrumbLabel(webhook_id, webhook?.name)

  return (
    <PageWebhookDetail
      webhook={webhook}
      isLoading={isLoading}
      tab={tab}
      tabs={tabs}
      name={current.name}
      endpoint={current.endpoint}
      description={current.description}
      headers={current.headers}
      subscribers={current.subscribers}
      errors={errors}
      dirtyCount={dirtyCount}
      onNameChange={(v) => patch({ name: v })}
      onEndpointChange={(v) => patch({ endpoint: v })}
      onDescriptionChange={(v) => patch({ description: v })}
      onHeadersChange={(next) => patch({ headers: next })}
      onSubscribersChange={(next) => patch({ subscribers: next })}
      onBack={() => navigate(NEXT_WEBHOOKS_URL(realm))}
      onDiscard={() => setDraft(pristine)}
      onSave={save}
      onDelete={handleDelete}
    />
  )
}
