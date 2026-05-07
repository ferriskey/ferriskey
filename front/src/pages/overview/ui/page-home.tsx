import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  ChartConfig,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from '@/components/ui/chart'
import { Heading } from '@/components/ui/heading'
import { Skeleton } from '@/components/ui/skeleton'
import { REALM_URL, RouterParams } from '@/routes/router'
import { OVERVIEW_URL } from '@/routes/sub-router/client.router'
import { ROLE_OVERVIEW_URL } from '@/routes/sub-router/role.router'
import { USER_OVERVIEW_URL } from '@/routes/sub-router/user.router'
import { PageHomeData, QuickAccessItem } from '@/types'
import {
  ArrowRight,
  Compass,
  Fingerprint,
  KeyRound,
  Mail,
  Pyramid,
  RotateCcw,
  ShieldUser,
  UserPlus,
  Users,
} from 'lucide-react'
import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { Area, AreaChart, CartesianGrid, XAxis, YAxis } from 'recharts'
import userStore from '@/store/user.store'

const growthConfig = {
  users: {
    label: 'Users',
    color: 'oklch(0.696 0.17 162.48)',
  },
} satisfies ChartConfig

interface PageHomeProps {
  data: PageHomeData
}

interface KpiCardData {
  title: string
  value: string | number
  hint: string
  icon: React.ComponentType<{ className?: string }>
  color: string
  bg: string
}

interface CapabilityItem {
  key: string
  label: string
  description: string
  icon: React.ComponentType<{ className?: string }>
  enabled: boolean
}

const DAYS_WINDOW = 30

function buildGrowthSeries(users: { created_at: string }[]) {
  const today = new Date()
  today.setHours(0, 0, 0, 0)

  const buckets: { date: string; key: string; users: number }[] = []
  const counts = new Map<string, number>()

  for (let i = DAYS_WINDOW - 1; i >= 0; i--) {
    const d = new Date(today)
    d.setDate(today.getDate() - i)
    const key = d.toISOString().slice(0, 10)
    counts.set(key, 0)
    buckets.push({
      date: d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' }),
      key,
      users: 0,
    })
  }

  for (const u of users) {
    const d = new Date(u.created_at)
    if (Number.isNaN(d.getTime())) continue
    const key = d.toISOString().slice(0, 10)
    if (counts.has(key)) counts.set(key, (counts.get(key) ?? 0) + 1)
  }

  return buckets.map((b) => ({ ...b, users: counts.get(b.key) ?? 0 }))
}

export default function PageHome({ data }: PageHomeProps) {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const currentUser = userStore((s) => s.user)
  const { clients, users, roles, flowStats, realmSettings, isLoading } = data

  const greetingName = useMemo(() => {
    if (!currentUser) return null
    const fromName = currentUser.name?.trim().split(/\s+/)[0]
    return fromName || currentUser.preferred_username || null
  }, [currentUser])

  const go = (url: string) => {
    if (!realm_name) return
    navigate(`${REALM_URL(realm_name)}${url}`)
  }

  const kpis: KpiCardData[] = useMemo(() => {
    const totalFlows = flowStats?.total ?? 0
    const successFlows = flowStats?.success_count ?? 0
    const verifiedUsers = users.filter((u) => u.email_verified).length
    const activeClients = clients.filter((c) => c.enabled).length
    const pct = (n: number, d: number) => (d > 0 ? Math.round((n / d) * 100) : 0)

    return [
      {
        title: 'Users',
        value: users.length,
        hint: users.length > 0 ? `${pct(verifiedUsers, users.length)}% email verified` : 'No users yet',
        icon: Users,
        color: 'text-emerald-500',
        bg: 'bg-emerald-500/10',
      },
      {
        title: 'Clients',
        value: clients.length,
        hint: clients.length > 0 ? `${activeClients} active` : 'No clients yet',
        icon: Pyramid,
        color: 'text-blue-500',
        bg: 'bg-blue-500/10',
      },
      {
        title: 'Roles',
        value: roles.length,
        hint: 'Permissions & policies',
        icon: ShieldUser,
        color: 'text-violet-500',
        bg: 'bg-violet-500/10',
      },
      {
        title: 'Auth flows',
        value: totalFlows,
        hint: totalFlows > 0 ? `${pct(successFlows, totalFlows)}% success` : 'No traces yet',
        icon: Compass,
        color: 'text-amber-500',
        bg: 'bg-amber-500/10',
      },
    ]
  }, [clients, users, roles, flowStats])

  const growthSeries = useMemo(() => buildGrowthSeries(users), [users])
  const newUsersWindow = useMemo(
    () => growthSeries.reduce((acc, b) => acc + b.users, 0),
    [growthSeries],
  )

  const capabilities: CapabilityItem[] = useMemo(
    () => [
      {
        key: 'passkey',
        label: 'Passkey',
        description: 'WebAuthn passwordless',
        icon: Fingerprint,
        enabled: !!realmSettings?.passkey_enabled,
      },
      {
        key: 'magic_link',
        label: 'Magic links',
        description: 'Email-based sign-in',
        icon: Mail,
        enabled: !!realmSettings?.magic_link_enabled,
      },
      {
        key: 'registration',
        label: 'Self registration',
        description: 'Public sign-up',
        icon: UserPlus,
        enabled: !!realmSettings?.user_registration_enabled,
      },
      {
        key: 'forgot_password',
        label: 'Password reset',
        description: 'Forgot password flow',
        icon: KeyRound,
        enabled: !!realmSettings?.forgot_password_enabled,
      },
      {
        key: 'remember_me',
        label: 'Remember me',
        description: 'Persistent sessions',
        icon: RotateCcw,
        enabled: !!realmSettings?.remember_me_enabled,
      },
      {
        key: 'compass',
        label: 'Compass tracing',
        description: 'Auth flow analytics',
        icon: Compass,
        enabled: !!realmSettings?.compass_enabled,
      },
    ],
    [realmSettings],
  )

  const quickAccess: QuickAccessItem[] = [
    {
      title: 'Users',
      icon: Users,
      url: `/users${USER_OVERVIEW_URL}`,
      description: 'Invite, edit and audit accounts',
    },
    {
      title: 'Clients',
      icon: Pyramid,
      url: `/clients${OVERVIEW_URL}`,
      description: 'Configure OAuth applications',
    },
    {
      title: 'Roles',
      icon: ShieldUser,
      url: `/roles${ROLE_OVERVIEW_URL}`,
      description: 'Define permissions',
    },
    {
      title: 'Compass',
      icon: Compass,
      url: '/compass/overview',
      description: 'Trace authentication flows',
    },
  ]

  return (
    <div className='min-h-screen bg-gradient-to-b from-background via-background to-muted/40'>
      <div className='flex flex-col gap-12 p-6 md:p-10'>
        {/* Header */}
        <div className='flex flex-col gap-2 pt-2 border-l-2 border-primary pl-4'>
          <div className='flex items-center gap-3 text-xs font-medium uppercase tracking-wider text-muted-foreground'>
            <img src='/logo_ferriskey.png' alt='' className='h-4 w-4 opacity-80' />
            <span>FerrisKey · Identity & Access Management</span>
            {realm_name && (
              <Badge variant='outline' className='rounded-md border-primary/30 text-primary uppercase tracking-wider'>
                {realm_name}
              </Badge>
            )}
          </div>
          <Heading size={2} weight='medium'>
            Welcome back{greetingName ? <>, <span className='text-primary'>{greetingName}</span></> : ''}
          </Heading>
        </div>

        {/* KPI Cards */}
        <div className='grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6'>
          {isLoading
            ? Array.from({ length: 4 }).map((_, i) => (
                <Card key={`kpi-skel-${i}`} className='rounded-md'>
                  <CardHeader className='flex flex-row items-center justify-between pb-3 pt-7 px-7'>
                    <Skeleton className='h-4 w-20 rounded-md' />
                    <Skeleton className='h-10 w-10 rounded-md' />
                  </CardHeader>
                  <CardContent className='space-y-3 pb-7 px-7'>
                    <Skeleton className='h-9 w-16 rounded-md' />
                    <Skeleton className='h-3 w-28 rounded-md' />
                  </CardContent>
                </Card>
              ))
            : kpis.map((k) => (
                <Card key={k.title} className='rounded-md transition hover:border-primary/30 hover:shadow-sm'>
                  <CardHeader className='flex flex-row items-center justify-between pb-3 pt-7 px-7'>
                    <CardTitle className='text-sm font-medium text-muted-foreground'>{k.title}</CardTitle>
                    <div className={`h-10 w-10 rounded-md ${k.bg} flex items-center justify-center`}>
                      <k.icon className={`h-5 w-5 ${k.color}`} />
                    </div>
                  </CardHeader>
                  <CardContent className='pb-7 px-7'>
                    <div className='text-3xl font-bold tabular-nums'>{k.value}</div>
                    <p className='text-xs text-muted-foreground mt-2'>{k.hint}</p>
                  </CardContent>
                </Card>
              ))}
        </div>

        {/* User growth + Realm capabilities */}
        <div className='grid grid-cols-1 lg:grid-cols-5 gap-6'>
          {/* User growth */}
          <Card className='rounded-md lg:col-span-3 flex flex-col'>
            <CardHeader className='pb-2 pt-7 px-7'>
              <div className='flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between'>
                <div>
                  <CardTitle className='text-base'>User growth</CardTitle>
                  <CardDescription>New accounts over the last {DAYS_WINDOW} days</CardDescription>
                </div>
                {!isLoading && (
                  <div className='text-right'>
                    <div className='text-2xl font-bold tabular-nums'>+{newUsersWindow}</div>
                    <p className='text-xs text-muted-foreground'>new users</p>
                  </div>
                )}
              </div>
            </CardHeader>
            <CardContent className='flex-1 pb-7 px-7'>
              {isLoading ? (
                <Skeleton className='h-[240px] w-full rounded-md' />
              ) : users.length === 0 ? (
                <div className='flex h-[240px] items-center justify-center text-sm text-muted-foreground'>
                  No users yet
                </div>
              ) : (
                <ChartContainer config={growthConfig} className='h-[240px] w-full'>
                  <AreaChart data={growthSeries} margin={{ left: 4, right: 4, top: 8, bottom: 0 }}>
                    <defs>
                      <linearGradient id='growthFill' x1='0' y1='0' x2='0' y2='1'>
                        <stop offset='0%' stopColor='var(--color-users)' stopOpacity={0.4} />
                        <stop offset='100%' stopColor='var(--color-users)' stopOpacity={0.02} />
                      </linearGradient>
                    </defs>
                    <CartesianGrid vertical={false} strokeDasharray='3 3' className='stroke-border/50' />
                    <XAxis
                      dataKey='date'
                      tickLine={false}
                      axisLine={false}
                      tickMargin={10}
                      minTickGap={28}
                      tick={{ fontSize: 11 }}
                    />
                    <YAxis
                      tickLine={false}
                      axisLine={false}
                      width={28}
                      tick={{ fontSize: 11 }}
                      allowDecimals={false}
                    />
                    <ChartTooltip cursor={false} content={<ChartTooltipContent indicator='line' />} />
                    <Area
                      type='monotone'
                      dataKey='users'
                      stroke='var(--color-users)'
                      strokeWidth={2}
                      fill='url(#growthFill)'
                    />
                  </AreaChart>
                </ChartContainer>
              )}
            </CardContent>
          </Card>

          {/* Realm capabilities */}
          <Card className='rounded-md lg:col-span-2 flex flex-col'>
            <CardHeader className='pb-2 pt-7 px-7'>
              <CardTitle className='text-base'>Realm capabilities</CardTitle>
              <CardDescription>Authentication features enabled</CardDescription>
            </CardHeader>
            <CardContent className='flex-1 pb-7 px-7 space-y-2'>
              {isLoading
                ? Array.from({ length: 6 }).map((_, i) => (
                    <Skeleton key={`cap-skel-${i}`} className='h-12 w-full rounded-md' />
                  ))
                : capabilities.map((c) => (
                    <div
                      key={c.key}
                      className={`relative flex items-center gap-3 rounded-md border-l-2 px-3 py-2.5 transition ${
                        c.enabled
                          ? 'border-l-emerald-500 bg-emerald-500/[0.04] border-y border-r border-emerald-500/15'
                          : 'border-l-muted-foreground/30 bg-muted/30 border-y border-r border-dashed border-border'
                      }`}
                    >
                      <div
                        className={`h-9 w-9 rounded-md flex items-center justify-center ${
                          c.enabled
                            ? 'bg-emerald-500/15 text-emerald-600'
                            : 'bg-muted text-muted-foreground/60'
                        }`}
                      >
                        <c.icon className='h-4 w-4' />
                      </div>
                      <div className='flex-1 min-w-0'>
                        <p
                          className={`text-sm font-medium ${
                            c.enabled ? 'text-foreground' : 'text-muted-foreground'
                          }`}
                        >
                          {c.label}
                        </p>
                        <p
                          className={`text-xs truncate ${
                            c.enabled ? 'text-muted-foreground' : 'text-muted-foreground/60'
                          }`}
                        >
                          {c.description}
                        </p>
                      </div>
                      {c.enabled ? (
                        <span className='inline-flex items-center gap-1 text-[11px] font-semibold uppercase tracking-wide text-emerald-600'>
                          <span className='relative flex h-2 w-2'>
                            <span className='absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500/60' />
                            <span className='relative inline-flex h-2 w-2 rounded-full bg-emerald-500' />
                          </span>
                          Active
                        </span>
                      ) : (
                        <span className='text-[11px] font-medium uppercase tracking-wide text-muted-foreground/70'>
                          Disabled
                        </span>
                      )}
                    </div>
                  ))}
            </CardContent>
          </Card>
        </div>

        {/* Quick access */}
        <div className='space-y-6'>
          <div className='space-y-1'>
            <h2 className='text-lg font-medium'>Get started</h2>
            <p className='text-sm text-muted-foreground'>Jump into a workspace</p>
          </div>
          <div className='grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4'>
            {quickAccess.map((q) => (
              <button
                key={q.title}
                onClick={() => go(q.url)}
                className='group flex flex-col items-start gap-4 border border-border bg-card/40 p-6 text-left transition hover:border-primary/30 hover:bg-muted/60 hover:shadow-sm'
              >
                <div className='h-11 w-11 bg-primary/10 text-primary flex items-center justify-center'>
                  <q.icon className='h-5 w-5' />
                </div>
                <div className='space-y-1'>
                  <p className='text-base font-medium group-hover:text-primary transition-colors'>
                    {q.title}
                  </p>
                  <p className='text-sm text-muted-foreground'>{q.description}</p>
                </div>
                <ArrowRight className='h-4 w-4 text-muted-foreground transition-all group-hover:translate-x-1 group-hover:text-primary mt-auto' />
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
