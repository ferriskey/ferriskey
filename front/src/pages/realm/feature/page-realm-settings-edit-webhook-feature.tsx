import { Form } from '@/components/ui/form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import PageRealmSettingsEditWebhook from '../ui/page-realm-settings-edit-webhook'
import { UpdateWebhookSchema, updateWebhookValidator } from '../validators'
import { useGetWebhook, useUpdateWebhook } from '@/api/webhook.api'
import { useEffect, useMemo, useState } from 'react'
import { Schemas } from '@/api/api.client'
import WebhookTrigger = Schemas.WebhookTrigger
import { getWebhookCategoriesForUI } from '@/utils/webhook-utils'
import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'

export default function PageRealmSettingsEditWebhookFeature() {
  const { realm_name, webhook_id } = useParams<RouterParams>()
  const { data: webhook } = useGetWebhook({ realm: realm_name, webhookId: webhook_id! })
  const { mutate: updateWebhook, data: responseData } = useUpdateWebhook()
  const navigate = useNavigate()
  const webhookCategories = getWebhookCategoriesForUI()

  const initialTriggers = useMemo(
    () => webhook?.subscribers.map((s) => s.name) ?? [],
    [webhook]
  )
  const [selectedTriggers, setSelectedTriggers] = useState<WebhookTrigger[]>([])
  const triggers = selectedTriggers.length > 0 || !webhook ? selectedTriggers : initialTriggers

  const webhookValues: UpdateWebhookSchema | undefined = webhook
    ? {
        name: webhook.name ?? '',
        description: webhook.description ?? '',
        endpoint: webhook.endpoint,
        subscribers: initialTriggers,
        headers: Object.entries(webhook.headers).map(([key, value]) => ({ key, value })),
      }
    : undefined

  const form = useForm<UpdateWebhookSchema>({
    resolver: zodResolver(updateWebhookValidator),
    mode: 'all',
    defaultValues: {
      name: '',
      description: '',
      endpoint: '',
      subscribers: [],
    },
    values: webhookValues,
  })

  const handleTriggerToggle = (trigger: WebhookTrigger) => {
    const current = triggers
    const newTriggers = current.includes(trigger)
      ? current.filter((t) => t !== trigger)
      : [...current, trigger]

    setSelectedTriggers(newTriggers)
    form.setValue('subscribers', newTriggers, { shouldDirty: true, shouldValidate: true })
  }

  const isTriggerSelected = (trigger: WebhookTrigger) => {
    return triggers.includes(trigger)
  }

  const onSubmit = form.handleSubmit((data) => {
    if (!realm_name || !webhook_id) return

    const headers: Record<string, string> = {}
    if (data.headers) {
      data.headers.forEach((header) => {
        headers[header.key] = header.value
      })
    }

    updateWebhook({
      body: {
        description: data.description,
        endpoint: data.endpoint,
        name: data.name,
        subscribers: data.subscribers as WebhookTrigger[],
        headers,
      },
      path: {
        realm_name,
        webhook_id,
      },
    }, {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      onError: (error: any) => {
        const errors = error?.data?.errors
        if (Array.isArray(errors) && errors.length > 0) {
          errors.forEach(({ message, field }: { message: string; field: string }) => {
            const fieldName = field as keyof UpdateWebhookSchema
            form.setError(fieldName, { message })
          })
        } else {
          toast.error(error?.message ?? 'Failed to update webhook')
        }
      }
    })
  })

  const handleBack = () => {
    navigate(`/realms/${realm_name}/realm-settings/webhooks`)
  }

  useEffect(() => {
    if (responseData) {
      navigate(`/realms/${realm_name}/realm-settings/webhooks`)
      toast.success('Webhook updated successfully')
    }
  }, [responseData, navigate, realm_name])

  return (
    <Form {...form}>
      <PageRealmSettingsEditWebhook
        webhoobCategories={webhookCategories}
        handleTriggerToggle={handleTriggerToggle}
        isTriggerSelected={isTriggerSelected}
        onSubmit={onSubmit}
        handleBack={handleBack}
      />
    </Form>
  )
}
