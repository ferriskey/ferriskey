import { Heading } from '@/components/ui/heading'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { ChartContainer, ChartTooltip, ChartTooltipContent } from '@/components/ui/chart'
import { REALM_URL, RouterParams } from '@/routes/router'
import { OVERVIEW_URL } from '@/routes/sub-router/client.router'
import { ROLE_OVERVIEW_URL } from '@/routes/sub-router/role.router'
import { USER_OVERVIEW_URL } from '@/routes/sub-router/user.router'
import {
  Pyramid,
  ShieldUser,
  Users,
  Activity,
  Clock,
  TrendingUp,
} from 'lucide-react'
import { useNavigate, useParams } from 'react-router-dom'
import { Skeleton } from '@/components/ui/skeleton'
import { useMemo } from 'react'
import { PieChart, Pie, Cell, ResponsiveContainer, BarChart, Bar, XAxis, CartesianGrid, Legend } from 'recharts'
import { PageHomeData, HomeMetrics, ChartDataItem, HomeChartConfig, QuickAccessItem } from '@/types'

const quickAccessItems: QuickAccessItem[] = [
  {
    title: 'Clients',
    icon: Pyramid,
    url: `/clients${OVERVIEW_URL}`,
    description: 'Manage OAuth clients'
  },
  {
    title: 'Users',
    icon: Users,
    url: `/users${USER_OVERVIEW_URL}`,
    description: 'User management'
  },
  {
    title: 'Roles',
    icon: ShieldUser,
    url: `/roles${ROLE_OVERVIEW_URL}`,
    description: 'Role & permissions'
  },
]

const chartConfig: HomeChartConfig = {
  clients: {
    label: 'Clients',
    color: '#3b82f6',
  },
  users: {
    label: 'Users',
    color: '#10b981',
  },
  roles: {
    label: 'Roles',
    color: '#8b5cf6',
  },
}

const barChartConfig = {
  active: {
    label: 'Active',
    color: '#10b981',
  },
  inactive: {
    label: 'Inactive',
    color: '#6b7280',
  },
}

interface PageHomeProps {
  data: PageHomeData
}

export default function PageHome({ data }: PageHomeProps) {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()

  const { clients, users, roles, isLoading } = data

  const handleClick = (url: string) => {
    if (!realm_name) return
    navigate(`${REALM_URL(realm_name)}${url}`)
  }

  // Calculate real-time metrics from passed data
  const metrics: HomeMetrics = useMemo(() => {
    const totalClients = clients?.length || 0
    const totalUsers = users?.length || 0
    const totalRoles = roles?.length || 0

    const activeClients = clients?.filter(client => client.enabled)?.length || 0
    const serviceAccountClients = clients?.filter(client => client.service_account_enabled)?.length || 0

    return {
      totalClients,
      totalUsers,
      totalRoles,
      activeClients,
      serviceAccountClients,
      clientsActivePercentage: totalClients > 0 ? Math.round((activeClients / totalClients) * 100) : 0,
    }
  }, [clients, users, roles])

  const chartData: ChartDataItem[] = useMemo(() => [
    { name: 'Clients', value: metrics.totalClients, fill: '#3b82f6' },
    { name: 'Users', value: metrics.totalUsers, fill: '#10b981' },
    { name: 'Roles', value: metrics.totalRoles, fill: '#8b5cf6' },
  ], [metrics])

  const barChartData = useMemo(() => [
    { name: 'Active', active: metrics.activeClients, inactive: metrics.totalClients - metrics.activeClients },
  ], [metrics])

  return (
    <div className='flex flex-col gap-8 p-6 md:p-10 container mx-auto max-w-7xl'>
      {/* Header Section */}
      <div className='flex flex-col gap-6'>
        <div className='flex items-center gap-3'>
          <img src='/logo_ferriskey.png' alt='FerrisKey Logo' className='h-12' />
          <div className='flex flex-col'>
            <Heading weight='medium'>Welcome to FerrisKey</Heading>
            <span className='text-sm text-muted-foreground'>
              Identity and Access Management Platform
            </span>
          </div>
        </div>

        <div className='flex items-center gap-2'>
          <span className='text-sm text-muted-foreground'>Current realm:</span>
          <Badge variant='secondary' className='bg-primary/10 text-primary'>
            {realm_name}
          </Badge>
        </div>
      </div>

      <Separator />

      {/* Real-time Metrics Dashboard */}
      <div className='space-y-6'>
        <div className='flex items-center gap-2'>
          <Activity className='size-5 text-primary' />
          <Heading size={3} weight='medium'>
            System Overview
          </Heading>
        </div>

        {isLoading ? (
          // Loading skeleton for metrics cards
          <div className='grid sm:grid-cols-2 lg:grid-cols-4 gap-6'>
            {Array.from({ length: 4 }).map((_, i) => (
              <Card key={i}>
                <CardHeader className='pb-3'>
                  <div className='flex items-center justify-between'>
                    <Skeleton className='h-4 w-24' />
                    <Skeleton className='h-4 w-4' />
                  </div>
                  <Skeleton className='h-8 w-16' />
                </CardHeader>
              </Card>
            ))}
          </div>
        ) : (
          <div className='grid sm:grid-cols-2 lg:grid-cols-4 gap-6'>
            {/* Total Resources Cards */}
            <Card className='hover:shadow-md transition-shadow'>
              <CardHeader className='pb-3'>
                <div className='flex items-center justify-between'>
                  <CardTitle className='text-sm font-medium text-muted-foreground'>
                    Total Clients
                  </CardTitle>
                  <Pyramid className='size-4 text-blue-500' />
                </div>
                <CardDescription className='text-2xl font-bold text-foreground'>
                  {metrics.totalClients}
                </CardDescription>
              </CardHeader>
            </Card>

            <Card className='hover:shadow-md transition-shadow'>
              <CardHeader className='pb-3'>
                <div className='flex items-center justify-between'>
                  <CardTitle className='text-sm font-medium text-muted-foreground'>
                    Total Users
                  </CardTitle>
                  <Users className='size-4 text-green-500' />
                </div>
                <CardDescription className='text-2xl font-bold text-foreground'>
                  {metrics.totalUsers}
                </CardDescription>
              </CardHeader>
            </Card>

            <Card className='hover:shadow-md transition-shadow'>
              <CardHeader className='pb-3'>
                <div className='flex items-center justify-between'>
                  <CardTitle className='text-sm font-medium text-muted-foreground'>
                    Total Roles
                  </CardTitle>
                  <ShieldUser className='size-4 text-purple-500' />
                </div>
                <CardDescription className='text-2xl font-bold text-foreground'>
                  {metrics.totalRoles}
                </CardDescription>
              </CardHeader>
            </Card>

            <Card className='hover:shadow-md transition-shadow'>
              <CardHeader className='pb-3'>
                <div className='flex items-center justify-between'>
                  <CardTitle className='text-sm font-medium text-muted-foreground'>
                    Active Clients
                  </CardTitle>
                  <TrendingUp className='size-4 text-orange-500' />
                </div>
                <CardDescription className='text-2xl font-bold text-foreground'>
                  {metrics.clientsActivePercentage}%
                </CardDescription>
              </CardHeader>
            </Card>
          </div>
        )}

        {/* Charts Section */}
        {!isLoading && (
          <div className='space-y-6 mt-8'>
            <div className='grid grid-cols-1 lg:grid-cols-2 gap-6'>
              {/* Resource Distribution Chart */}
              <Card>
                <CardHeader>
                  <CardTitle className='text-base'>Resource Distribution</CardTitle>
                  <CardDescription>
                    Overview of your realm resources
                  </CardDescription>
                </CardHeader>
                <CardContent className='p-0 overflow-hidden'>
                  <div className='p-4 sm:p-6'>
                    <ChartContainer config={chartConfig} className='h-[250px] sm:h-[300px] w-full'>
                      <ResponsiveContainer width='100%' height='100%' minWidth={0}>
                        <PieChart margin={{ top: 20, right: 20, left: 20, bottom: 60 }}>
                          <Pie
                            data={chartData}
                            cx='50%'
                            cy='45%'
                            innerRadius={45}
                            outerRadius={85}
                            paddingAngle={0}
                            dataKey='value'
                          >
                            {chartData.map((entry, index) => (
                              <Cell key={`cell-${index}`} fill={entry.fill} />
                            ))}
                          </Pie>
                          <ChartTooltip
                            cursor={false}
                            content={<ChartTooltipContent hideLabel />}
                          />
                          <Legend
                            verticalAlign='bottom'
                            height={36}
                            iconType='circle'
                            wrapperStyle={{
                              fontSize: '12px',
                              paddingTop: '20px'
                            }}
                          />
                        </PieChart>
                      </ResponsiveContainer>
                    </ChartContainer>
                  </div>
                </CardContent>
              </Card>

              {/* Client Status Chart */}
              <Card>
                <CardHeader>
                  <CardTitle className='text-base'>Client Status</CardTitle>
                  <CardDescription>
                    Active vs inactive clients
                  </CardDescription>
                </CardHeader>
                <CardContent className='p-0 overflow-hidden'>
                  <div className='p-4 sm:p-6'>
                    <ChartContainer config={barChartConfig} className='h-[200px] sm:h-[250px] w-full'>
                      <BarChart accessibilityLayer data={barChartData}>
                        <CartesianGrid vertical={false} />
                        <XAxis
                          dataKey='name'
                          tickLine={false}
                          tickMargin={10}
                          axisLine={false}
                        />
                        <ChartTooltip
                          cursor={false}
                          content={<ChartTooltipContent indicator='dashed' />}
                        />
                        <Bar dataKey='active' fill='#10b981' radius={4} />
                        <Bar dataKey='inactive' fill='#6b7280' radius={4} />
                      </BarChart>
                    </ChartContainer>
                  </div>
                </CardContent>
              </Card>
            </div>
          </div>
        )}
      </div>

      <Separator />

      {/* Enhanced Quick Access Cards with Hover Effects */}
      <div className='space-y-6'>
        <div className='flex items-center gap-2'>
          <Clock className='size-5 text-primary' />
          <Heading size={3} weight='medium'>
            Quick Access
          </Heading>
        </div>

        <div className='grid sm:grid-cols-2 lg:grid-cols-3 gap-6'>
          {quickAccessItems.map((item, index) => (
            <Card
              key={index}
              className='hover:shadow-lg hover:shadow-primary/10 transition-all duration-200 cursor-pointer group'
              onClick={() => handleClick(item.url)}
            >
              <CardHeader>
                <div className='flex items-center gap-3'>
                  <div className='p-2 rounded-lg bg-primary/10 group-hover:bg-primary/20 transition-colors'>
                    <item.icon className='size-5 text-primary' />
                  </div>
                  <div>
                    <CardTitle className='text-base group-hover:text-primary transition-colors'>
                      {item.title}
                    </CardTitle>
                    <CardDescription className='text-sm'>
                      {item.description}
                    </CardDescription>
                  </div>
                </div>
              </CardHeader>
            </Card>
          ))}
        </div>
      </div>
    </div>
  )
}
