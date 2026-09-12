import { useMemo } from 'react'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useGetSmtpConfig } from '@/api/smtp.api'
import { useGetWebhooks } from '@/api/webhook.api'
import { useRealmDirectory } from '@/hooks/use-realm-directory'
import { useWindowEvents } from './use-window-events'
import PageMessages from '../ui/page-messages'

const EMAIL_EVENTS = ['email_sent', 'email_not_sent'] as const

export default function PageMessagesFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const directory = useRealmDirectory(realm)
  const { events, isLoading, isError, truncated, windowDays, windowLimit } = useWindowEvents(
    realm,
    EMAIL_EVENTS
  )

  const { data: smtpConfig, isLoading: isLoadingSmtp } = useGetSmtpConfig({ realm })
  const { data: webhooksResponse } = useGetWebhooks({ realm })

  const webhooks = useMemo(() => webhooksResponse?.data ?? [], [webhooksResponse])

  return (
    <PageMessages
      events={events}
      webhooks={webhooks}
      smtpConfigured={Boolean(smtpConfig)}
      isLoading={isLoading}
      isLoadingSmtp={isLoadingSmtp}
      isError={isError}
      truncated={truncated}
      windowDays={windowDays}
      windowLimit={windowLimit}
      directory={directory}
    />
  )
}
