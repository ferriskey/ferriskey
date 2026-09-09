import { Section } from '@/components/kit'
import WorkInProgress from '@/components/work-in-progress'

export default function WebhookDeliveriesTab() {
  return (
    <Section
      title='Recent deliveries'
      description='The API exposes no delivery history: the outcome of each call is recorded on the webhook row and never returned.'
      contained={false}
    >
      <WorkInProgress />
    </Section>
  )
}
