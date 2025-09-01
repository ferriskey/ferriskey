import { Form } from '@/components/ui/form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import PageRealmSettingsCreateWebhook from '../ui/page-realm-settings-create-webhook'
import { CreateWebhookSchema, createWebhookValidator } from '../validators'

export default function PageRealmSettingsCreateWebhookFeature() {
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

  return (
    <Form {...form}>
      <PageRealmSettingsCreateWebhook />
    </Form>
  )
}
