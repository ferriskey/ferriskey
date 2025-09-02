import { Form } from '@/components/ui/form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import PageRealmSettingsCreateWebhook from '../ui/page-realm-settings-create-webhook'
import { CreateWebhookSchema, createWebhookValidator } from '../validators'
import { useCreateWebhook } from '@/api/webhook.api'
import { useEffect, useState } from 'react'
import { Schemas } from '@/api/api.client'
import WebhookTrigger = Schemas.WebhookTrigger
import { getWebhookCategoriesForUI } from '@/utils/webhook-utils'
import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'


export default function PageRealmSettingsCreateWebhookFeature() {
  const { mutate: createWebhook, data: responseData } = useCreateWebhook()
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const [selectedTriggers, setSelectedTriggers] = useState<WebhookTrigger[]>([])
  const webhookCategories = getWebhookCategoriesForUI()

  const handleTriggerToggle = (trigger: WebhookTrigger) => {
    setSelectedTriggers(prev => {
      const newTriggers = prev.includes(trigger)
        ? prev.filter(t => t !== trigger)
        : [...prev, trigger]

      // Mettre à jour le formulaire
      form.setValue('subscribers', newTriggers)
      return newTriggers
    })
  }

  const isTriggerSelected = (trigger: WebhookTrigger) => {
    return selectedTriggers.includes(trigger)
  }

  useEffect(() => {
    console.log(selectedTriggers)
  }, [selectedTriggers])

  const form = useForm<CreateWebhookSchema>({
    resolver: zodResolver(createWebhookValidator),
    mode: 'all',
    values: {
      name: '',
      description: '',
      endpoint: '',
      subscribers: [],
    },
  })

  const onSubmit = form.handleSubmit((data) => {
    if (!realm_name) return

    createWebhook({
      body: {
        description: data.description,
        endpoint: data.endpoint,
        name: data.name,
        subscribers: selectedTriggers
      },
      path: {
        realm_name
      }
    })
  })

  const handleBack = () => {
    navigate(`/realms/${realm_name}/realm-settings/webhooks`)
  }

  useEffect(() => {
    if (responseData) {
      navigate(`/realms/${realm_name}/realm-settings/webhooks`)
      toast.success('Webhook created successfully')
    }
  }, [responseData, navigate, realm_name])

  return (
    <Form {...form}>
      <PageRealmSettingsCreateWebhook
        webhoobCategories={webhookCategories}
        handleTriggerToggle={handleTriggerToggle}
        isTriggerSelected={isTriggerSelected}
        onSubmit={onSubmit}
        handleBack={handleBack}
      />
    </Form>
  )
}
