import { Navigate, Route, Routes } from 'react-router'
import PageWebhooksOverviewFeature from './feature/page-webhooks-overview-feature'
import PageCreateWebhookFeature from './feature/page-create-webhook-feature'
import PageWebhookDetailFeature from './feature/page-webhook-detail-feature'

export default function PageWebhooks() {
  return (
    <Routes>
      <Route index element={<PageWebhooksOverviewFeature />} />
      <Route path='create' element={<PageCreateWebhookFeature />} />
      <Route path=':webhook_id' element={<Navigate to='settings' replace />} />
      <Route path=':webhook_id/*' element={<PageWebhookDetailFeature />} />
    </Routes>
  )
}
