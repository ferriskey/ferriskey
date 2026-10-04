import { useMemo } from 'react'
import { useParams } from 'react-router'
import { RouterParams, WEBHOOKS_URL } from '@/routes/router'
import { useGetSmtpConfig } from '@/api/smtp.api'
import { useGetWebhooks } from '@/api/webhook.api'
import { eventUserIds, useRealmDirectory } from '@/hooks/use-realm-directory'
import { eventRoleIds } from '@/hooks/event-role-ids'
import { eventClientIds } from '@/hooks/event-client-ids'
import { detailString } from '../ui/event-journal'
import { useEventJournal } from './use-event-journal'
import { useWindowCount, useWindowEvents } from './use-window-events'
import PageMessages from '../ui/page-messages'

const EMAIL_EVENTS = ['email_sent', 'email_not_sent'] as const

const WEBHOOK_PREVIEW = 20

export default function PageMessagesFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { events, total, range, isLoading, isError, truncated, windowDays, windowLimit } =
    useWindowEvents(realm, EMAIL_EVENTS)
  const delivered = useWindowCount(realm, range, { event_types: 'email_sent' })
  const failed = useWindowCount(realm, range, { event_types: 'email_not_sent' })
  const journal = useEventJournal(realm, EMAIL_EVENTS)
  const known = useMemo(() => [...events, ...journal.events], [events, journal.events])
  const userIds = useMemo(
    () => [
      ...eventUserIds(known),
      ...known.flatMap((event) => detailString(event, 'user_id') ?? []),
    ],
    [known]
  )
  const roleIds = useMemo(() => eventRoleIds(known), [known])
  const clientIds = useMemo(() => eventClientIds(known), [known])
  const directory = useRealmDirectory(realm, userIds, roleIds, clientIds)

  const { data: smtpConfig, isLoading: isLoadingSmtp } = useGetSmtpConfig({ realm })
  const { data: webhooksResponse } = useGetWebhooks({ realm, query: { limit: WEBHOOK_PREVIEW } })

  const webhooks = useMemo(() => webhooksResponse?.data ?? [], [webhooksResponse])

  return (
    <PageMessages
      events={events}
      counts={{ delivered, failed, total }}
      journal={journal}
      webhooks={webhooks}
      webhookTotal={webhooksResponse?.metadata.total ?? 0}
      webhooksHref={WEBHOOKS_URL(realm)}
      smtpConfigured={Boolean(smtpConfig)}
      isLoading={isLoading}
      isLoadingSmtp={isLoadingSmtp}
      isError={isError || journal.isError}
      truncated={truncated}
      windowDays={windowDays}
      windowLimit={windowLimit}
      directory={directory}
    />
  )
}
