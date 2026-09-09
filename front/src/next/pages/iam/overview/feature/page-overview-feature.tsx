import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
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
  NEXT_CLIENTS_URL,
  NEXT_COMPASS_URL,
  NEXT_REALM_SETTINGS_URL,
  NEXT_ROLES_URL,
  NEXT_USERS_URL,
} from '@/next/routes'
import PageOverview, {
  type OverviewAlert,
  type OverviewCapability,
  type OverviewQuickLink,
} from '../ui/page-overview'
import type { OverviewEvent } from '../ui/overview-event-log'
import { cumulativeSeries } from '@/next/shared/cumulative-series'

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
      label: 'Users',
      value: users.length,
      hint: users.length > 0 ? `${pct(verifiedUsers, users.length)}% email verified` : 'no user yet',
      delta: newUsers > 0 ? newUsers : undefined,
      series: cumulativeSeries(users.map((u) => u.created_at)),
      tone: 'success',
    },
    {
      key: 'clients',
      label: 'Clients',
      value: clients.length,
      hint: clients.length > 0 ? `${activeClients} active` : 'no client yet',
      series: cumulativeSeries(clients.map((c) => c.created_at)),
      tone: 'info',
    },
    {
      key: 'roles',
      label: 'Roles',
      value: roles.length,
      hint: 'permissions & policies',
      series: cumulativeSeries(roles.map((r) => r.created_at)),
      tone: 'violet',
    },
    {
      key: 'flows',
      label: 'Auth flows',
      value: totalFlows,
      hint: totalFlows > 0 ? `${pct(successFlows, totalFlows)}% success` : 'no trace yet',
      series: measured(activity.map((day) => day.total_flows)),
      tone: 'amber',
    },
  ]

  const alerts: OverviewAlert[] = []

  if (failedFlows > 0) {
    alerts.push({
      key: 'failed-flows',
      tone: 'error',
      title: `${failedFlows} authentication flow${failedFlows > 1 ? 's' : ''} failed`,
      detail: `${failedFlows} of ${totalFlows} traced flows never issued a token.`,
      action: 'Inspect',
      onAction: () => navigate(NEXT_COMPASS_URL(realm)),
    })
  }

  if (disabledClients.length > 0) {
    alerts.push({
      key: 'disabled-clients',
      tone: 'warn',
      title: `${disabledClients.length} client${disabledClients.length > 1 ? 's are' : ' is'} disabled`,
      detail: `${disabledClients.map((c) => c.client_id).join(', ')} — no account can authenticate through ${disabledClients.length > 1 ? 'them' : 'it'}.`,
      action: 'Review',
      onAction: () => navigate(NEXT_CLIENTS_URL(realm)),
    })
  }

  if (settings?.email_verification_enabled && unverifiedUsers.length > 0) {
    alerts.push({
      key: 'unverified-users',
      tone: 'warn',
      title: `${unverifiedUsers.length} account${unverifiedUsers.length > 1 ? 's carry' : ' carries'} an unverified email`,
      detail: 'Email verification is required on this realm, so those accounts stay incomplete.',
      action: 'Review',
      onAction: () => navigate(NEXT_USERS_URL(realm)),
    })
  }

  if (settings && !settings.compass_enabled) {
    alerts.push({
      key: 'compass-off',
      tone: 'warn',
      title: 'Compass tracing is off',
      detail: 'Authentication flows are not recorded, so this page has no activity to show.',
      action: 'Enable',
      onAction: () => navigate(NEXT_REALM_SETTINGS_URL(realm)),
    })
  }

  const capabilities: OverviewCapability[] = [
    {
      key: 'passkey',
      label: 'Passkey',
      description: 'WebAuthn passwordless',
      enabled: Boolean(settings?.passkey_enabled),
    },
    {
      key: 'magic_link',
      label: 'Magic links',
      description: 'Email-based sign-in',
      enabled: Boolean(settings?.magic_link_enabled),
    },
    {
      key: 'registration',
      label: 'Self registration',
      description: 'Public sign-up',
      enabled: Boolean(settings?.user_registration_enabled),
    },
    {
      key: 'forgot_password',
      label: 'Password reset',
      description: 'Forgot password flow',
      enabled: Boolean(settings?.forgot_password_enabled),
    },
    {
      key: 'remember_me',
      label: 'Remember me',
      description: 'Persistent sessions',
      enabled: Boolean(settings?.remember_me_enabled),
    },
    {
      key: 'compass',
      label: 'Compass tracing',
      description: 'Auth flow analytics',
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
      label: 'Users',
      description: 'Invite, edit and audit accounts',
      href: NEXT_USERS_URL(realm),
      icon: Users,
    },
    {
      key: 'clients',
      label: 'Clients',
      description: 'Configure OAuth applications',
      href: NEXT_CLIENTS_URL(realm),
      icon: LayoutGrid,
    },
    {
      key: 'roles',
      label: 'Roles',
      description: 'Define permissions',
      href: NEXT_ROLES_URL(realm),
      icon: Shield,
    },
    {
      key: 'compass',
      label: 'Compass',
      description: 'Trace authentication flows',
      href: NEXT_COMPASS_URL(realm),
      icon: Compass,
    },
  ]

  return (
    <PageOverview
      realmTitle={realmResponse?.display_name || realm}
      realmName={realm}
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
        settings?.compass_enabled
          ? 'No authentication flow recorded over this window.'
          : 'Compass tracing is off — no activity is recorded for this realm.'
      }
      events={events}
      eventsEmptyLabel={
        settings?.compass_enabled
          ? 'No flow traced yet.'
          : 'Compass tracing is off — no flow is traced for this realm.'
      }
      eventsHref={NEXT_COMPASS_URL(realm)}
      quickLinks={quickLinks}
    />
  )
}
