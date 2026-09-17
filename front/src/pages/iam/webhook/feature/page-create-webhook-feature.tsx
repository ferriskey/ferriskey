import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useCreateWebhook } from '@/api/webhook.api'
import { RouterParams } from '@/routes/router'
import { createWebhookValidator } from '@/pages/iam/realm/validators'
import { Schemas } from '@/api/api.client'
import { WEBHOOKS_URL } from '@/routes/router'
import PageCreateWebhook from '../ui/page-create-webhook'
import type { WebhookHeader } from '../ui/webhook-headers-field'
import { apiErrorMessage, partitionFieldErrors, validationErrorsFrom } from '@/lib/api-error'

import WebhookTrigger = Schemas.WebhookTrigger

type WebhookField = 'name' | 'endpoint' | 'description'

const FORM_FIELDS: WebhookField[] = ['name', 'endpoint', 'description']

export default function PageCreateWebhookFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { mutate: createWebhook } = useCreateWebhook()

  const [name, setName] = useState('')
  const [endpoint, setEndpoint] = useState('')
  const [description, setDescription] = useState('')
  const [headers, setHeaders] = useState<WebhookHeader[]>([])
  const [subscribers, setSubscribers] = useState<WebhookTrigger[]>([])
  const [touched, setTouched] = useState<Partial<Record<WebhookField, boolean>>>({})
  const [serverErrors, setServerErrors] = useState<
    Partial<Record<WebhookField, string>>
  >({})

  const parsed = createWebhookValidator.safeParse({
    name,
    endpoint,
    description,
    subscribers,
    headers,
  })

  const errors: Partial<Record<WebhookField, string>> = { ...serverErrors }
  if (!parsed.success) {
    for (const issue of parsed.error.issues) {
      const field = issue.path[0] as WebhookField
      if (touched[field] && !errors[field]) errors[field] = issue.message
    }
  }

  const back = () => navigate(WEBHOOKS_URL(realm))

  const submit = () => {
    if (!realm_name || !parsed.success) return

    const flattened: Record<string, string> = {}
    for (const header of headers) flattened[header.key] = header.value

    createWebhook(
      {
        body: { name, description, endpoint, subscribers, headers: flattened },
        path: { realm_name },
      },
      {
        onSuccess: () => {
          toast.success('Webhook created successfully')
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

          toast.error(apiErrorMessage(error, 'Failed to create webhook'))
        },
      }
    )
  }

  const edit = (field: WebhookField, apply: () => void) => {
    setTouched((t) => ({ ...t, [field]: true }))
    setServerErrors((e) => ({ ...e, [field]: undefined }))
    apply()
  }

  return (
    <PageCreateWebhook
      name={name}
      endpoint={endpoint}
      description={description}
      headers={headers}
      subscribers={subscribers}
      errors={errors}
      canSubmit={parsed.success}
      onNameChange={(v) => edit('name', () => setName(v))}
      onEndpointChange={(v) => edit('endpoint', () => setEndpoint(v))}
      onDescriptionChange={(v) => edit('description', () => setDescription(v))}
      onHeadersChange={setHeaders}
      onSubscribersChange={setSubscribers}
      onBack={back}
      onSubmit={submit}
    />
  )
}
