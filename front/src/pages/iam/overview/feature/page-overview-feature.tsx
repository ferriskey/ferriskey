import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { Compass, LayoutGrid, Shield, Users } from 'lucide-react'
import { useGetClients } from '@/api/client.api'
import { useGetUsers } from '@/api/user.api'
import { useGetRoles } from '@/api/role.api'
import { useGetDailyActivityStats, useGetFlows, useGetStats } from '@/api/compass.api'
import { useGetRealm } from '@/api/realm.api'
import { RouterParams } from '@/routes/router'
import userStore from '@/store/user.store'
import type { Metric } from '@/components/kit'
import {
  CLIENTS_URL,
  COMPASS_URL,
  REALM_SETTINGS_URL,
  ROLES_URL,
  USERS_URL,
} from '@/routes/router'
import PageOverview, {
  type OverviewAlert,
  type OverviewCapability,
  type OverviewQuickLink,
} from '../ui/page-overview'
import type { OverviewEvent } from '../ui/overview-event-log'
import { cumulativeSeries } from '@/utils/cumulative-series'

const WINDOW_DAYS = 30
const EVENT_COUNT = 8

const pct = (n: number, d: number) => (d > 0 ? Math.round((n / d) * 100) : 0)

function windowDays() {
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  return Array.from({ length: WINDOW_DAYS }, (_, i) => {
    const day = new Date(today)
    day.setDate(today.getDate() - (WINDOW_DAYS - 1 - i))
    return day.getTime()
  })
}

function measured(series: number[]) {
  return series.length > 0 ? series : undefined
}

function createdInWindow(createdAt: string[]) {
  const [first] = windowDays()
  return createdAt.filter((value) => {
    const time = new Date(value).getTime()
    return !Number.isNaN(time) && time >= first
  }).length
}

export default function PageOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { t } = useTranslation('overview')
  const realm = realm_name ?? 'master'
  const currentUser = userStore((s) => s.user)

  const { data: realmResponse, isLoading: isLoadingRealm } = useGetRealm({ realm })
  const { data: clientsResponse, isLoading: isLoadingClients } = useGetClients({ realm })
  const { data: usersResponse, isLoading: isLoadingUsers } = useGetUsers({ realm })
  const { data: rolesResponse, isLoading: isLoadingRoles } = useGetRoles({ realm })
  const { data: statsResponse, isLoading: isLoadingStats } = useGetStats({ realm })

  const settings = realmResponse?.settings ?? null
  const compassRealm = settings?.compass_enabled ? realm : undefined

  const { data: activityResponse } = useGetDailyActivityStats({ realm: compassRealm })
  const { data: flowsResponse } = useGetFlows({ realm: compassRealm, limit: EVENT_COUNT })

  const clients = useMemo(() => clientsResponse?.data ?? [], [clientsResponse])
  const users = useMemo(() => usersResponse?.data ?? [], [usersResponse])
  const roles = useMemo(() => rolesResponse?.data ?? [], [rolesResponse])
  const activity = useMemo(() => activityResponse?.data ?? [], [activityResponse])
  const flows = useMemo(() => flowsResponse?.data ?? [], [flowsResponse])
  const flowStats = statsResponse?.data ?? null

  const greeting = useMemo(() => {
    if (!currentUser) return undefined
    const fromName = currentUser.name?.trim().split(/\s+/)[0]
    return fromName || currentUser.preferred_username || undefined
  }, [currentUser])

  const verifiedUsers = users.filter((u) => u.email_verified).length
  const activeClients = clients.filter((c) => c.enabled).length
  const disabledClients = clients.filter((c) => !c.enabled)
  const unverifiedUsers = users.filter((u) => !u.email_verified)
  const totalFlows = flowStats?.total ?? 0
  const successFlows = flowStats?.success_count ?? 0
  const failedFlows = flowStats?.failure_count ?? 0

  const newUsers = createdInWindow(users.map((u) => u.created_at))

  const metrics: Metric[] = [
    {
      key: 'users',
      label: t('metrics.users.label'),
      value: users.length,
      hint:
        users.length > 0
          ? t('metrics.users.hint', { percent: pct(verifiedUsers, users.length) })
          : t('metrics.users.empty_hint'),
      delta: newUsers > 0 ? newUsers : undefined,
      series: cumulativeSeries(users.map((u) => u.created_at)),
      tone: 'success',
    },
    {
      key: 'clients',
      label: t('metrics.clients.label'),
      value: clients.length,
      hint:
        clients.length > 0
          ? t('metrics.clients.hint', { active: activeClients })
          : t('metrics.clients.empty_hint'),
      series: cumulativeSeries(clients.map((c) => c.created_at)),
      tone: 'info',
    },
    {
      key: 'roles',
      label: t('metrics.roles.label'),
      value: roles.length,
      hint: t('metrics.roles.hint'),
      series: cumulativeSeries(roles.map((r) => r.created_at)),
      tone: 'violet',
    },
    {
      key: 'flows',
      label: t('metrics.flows.label'),
      value: totalFlows,
      hint:
        totalFlows > 0
          ? t('metrics.flows.hint', { percent: pct(successFlows, totalFlows) })
          : t('metrics.flows.empty_hint'),
      series: measured(activity.map((day) => day.total_flows)),
      tone: 'amber',
    },
  ]

  const alerts: OverviewAlert[] = []

  if (failedFlows > 0) {
    alerts.push({
      key: 'failed-flows',
      tone: 'error',
      title: t('alerts.failed_flows.title', { count: failedFlows }),
      detail: t('alerts.failed_flows.detail', { failed: failedFlows, total: totalFlows }),
      action: t('alerts.failed_flows.action'),
      onAction: () => navigate(COMPASS_URL(realm)),
    })
  }

  if (disabledClients.length > 0) {
    alerts.push({
      key: 'disabled-clients',
      tone: 'warn',
      title: t('alerts.disabled_clients.title', { count: disabledClients.length }),
      detail: t('alerts.disabled_clients.detail', {
        count: disabledClients.length,
        names: disabledClients.map((c) => c.client_id).join(', '),
      }),
      action: t('alerts.disabled_clients.action'),
      onAction: () => navigate(CLIENTS_URL(realm)),
    })
  }

  if (settings?.email_verification_enabled && unverifiedUsers.length > 0) {
    alerts.push({
      key: 'unverified-users',
      tone: 'warn',
      title: t('alerts.unverified_users.title', { count: unverifiedUsers.length }),
      detail: t('alerts.unverified_users.detail'),
      action: t('alerts.unverified_users.action'),
      onAction: () => navigate(USERS_URL(realm)),
    })
  }

  if (settings && !settings.compass_enabled) {
    alerts.push({
      key: 'compass-off',
      tone: 'warn',
      title: t('alerts.compass_off.title'),
      detail: t('alerts.compass_off.detail'),
      action: t('alerts.compass_off.action'),
      onAction: () => navigate(REALM_SETTINGS_URL(realm)),
    })
  }

  const capabilities: OverviewCapability[] = [
    {
      key: 'passkey',
      label: t('capabilities.passkey.label'),
      description: t('capabilities.passkey.description'),
      enabled: Boolean(settings?.passkey_enabled),
    },
    {
      key: 'magic_link',
      label: t('capabilities.magic_link.label'),
      description: t('capabilities.magic_link.description'),
      enabled: Boolean(settings?.magic_link_enabled),
    },
    {
      key: 'registration',
      label: t('capabilities.registration.label'),
      description: t('capabilities.registration.description'),
      enabled: Boolean(settings?.user_registration_enabled),
    },
    {
      key: 'forgot_password',
      label: t('capabilities.forgot_password.label'),
      description: t('capabilities.forgot_password.description'),
      enabled: Boolean(settings?.forgot_password_enabled),
    },
    {
      key: 'remember_me',
      label: t('capabilities.remember_me.label'),
      description: t('capabilities.remember_me.description'),
      enabled: Boolean(settings?.remember_me_enabled),
    },
    {
      key: 'compass',
      label: t('capabilities.compass.label'),
      description: t('capabilities.compass.description'),
      enabled: Boolean(settings?.compass_enabled),
    },
  ]

  const events: OverviewEvent[] = flows.map((flow) => {
    const client = clients.find((c) => c.id === flow.client_id || c.client_id === flow.client_id)
    const user = users.find((u) => u.id === flow.user_id)
    return {
      id: flow.id,
      status: flow.status,
      grantType: flow.grant_type,
      client: client?.client_id ?? flow.client_id ?? undefined,
      user: user?.username ?? undefined,
      startedAt: flow.started_at,
      durationMs: flow.duration_ms,
    }
  })

  const quickLinks: OverviewQuickLink[] = [
    {
      key: 'users',
      label: t('quick_links.users.label'),
      description: t('quick_links.users.description'),
      href: USERS_URL(realm),
      icon: Users,
    },
    {
      key: 'clients',
      label: t('quick_links.clients.label'),
      description: t('quick_links.clients.description'),
      href: CLIENTS_URL(realm),
      icon: LayoutGrid,
    },
    {
      key: 'roles',
      label: t('quick_links.roles.label'),
      description: t('quick_links.roles.description'),
      href: ROLES_URL(realm),
      icon: Shield,
    },
    {
      key: 'compass',
      label: t('quick_links.compass.label'),
      description: t('quick_links.compass.description'),
      href: COMPASS_URL(realm),
      icon: Compass,
    },
  ]

  return (
    <PageOverview
      realmTitle={realmResponse?.display_name || realm}
      greeting={greeting}
      isLoading={
        isLoadingRealm ||
        isLoadingClients ||
        isLoadingUsers ||
        isLoadingRoles ||
        isLoadingStats
      }
      metrics={metrics}
      alerts={alerts}
      capabilities={capabilities}
      activity={activity}
      activityWindowDays={WINDOW_DAYS}
      activityEmptyLabel={
        settings?.compass_enabled ? t('activity.empty') : t('activity.empty_compass_off')
      }
      events={events}
      eventsEmptyLabel={
        settings?.compass_enabled ? t('events.empty') : t('events.empty_compass_off')
      }
      eventsHref={COMPASS_URL(realm)}
      quickLinks={quickLinks}
    />
  )
}
