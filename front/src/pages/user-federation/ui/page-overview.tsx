import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Filters, Filter, FilterFieldsConfig } from '@/components/ui/filters'
import { useState, useMemo } from 'react'

import {
  Database,
  Settings,
  Users,
  CheckCircle,
  Clock,
  Building2,
  Server,
  Wrench,
  Key,
  Cog,
  RefreshCw,
  ChevronRight,
  CircleDot,
} from 'lucide-react'

const PROVIDER_ICONS = {
  'Corporate LDAP': Building2,
  'Active Directory': Server,
  'Development LDAP': Wrench,
  'Legacy Kerberos': Key,
  'Custom User Storage': Cog,
}

interface Provider {
  name: string
  type: string
  status: string
  users: number
  lastSync: string
  connection: string
  priority: string
}

const PROVIDERS_DATA: Provider[] = [
  {
    name: 'Corporate LDAP',
    type: 'LDAP',
    status: 'active',
    users: 1456,
    lastSync: '5 minutes ago',
    connection: 'ldap.company.local:389',
    priority: 'Primary',
  },
  {
    name: 'Active Directory',
    type: 'LDAP',
    status: 'active',
    users: 892,
    lastSync: '10 minutes ago',
    connection: 'ad.company.com:636',
    priority: 'Secondary',
  },
  {
    name: 'Development LDAP',
    type: 'LDAP',
    status: 'syncing',
    users: 234,
    lastSync: 'Syncing now...',
    connection: 'dev-ldap.local:389',
    priority: 'Development',
  },
  {
    name: 'Legacy Kerberos',
    type: 'Kerberos',
    status: 'inactive',
    users: 156,
    lastSync: '2 days ago',
    connection: 'kerberos.legacy.local',
    priority: 'Legacy',
  },
  {
    name: 'Custom User Storage',
    type: 'Custom',
    status: 'active',
    users: 109,
    lastSync: '1 hour ago',
    connection: 'api.users.company.com',
    priority: 'Custom',
  }
]

export default function PageOverview() {
  const [filters, setFilters] = useState<Filter[]>([])

  const filterFields: FilterFieldsConfig = [
    {
      key: 'name',
      label: 'Provider Name',
      type: 'text',
    },
    {
      key: 'type',
      label: 'Type',
      type: 'select',
      options: [
        { value: 'LDAP', label: 'LDAP' },
        { value: 'Kerberos', label: 'Kerberos' },
        { value: 'Custom', label: 'Custom' },
      ],
    },
    {
      key: 'status',
      label: 'Status',
      type: 'select',
      options: [
        { value: 'active', label: 'Active' },
        { value: 'syncing', label: 'Syncing' },
        { value: 'inactive', label: 'Inactive' },
      ],
    },
    {
      key: 'priority',
      label: 'Priority',
      type: 'select',
      options: [
        { value: 'Primary', label: 'Primary' },
        { value: 'Secondary', label: 'Secondary' },
        { value: 'Development', label: 'Development' },
        { value: 'Legacy', label: 'Legacy' },
        { value: 'Custom', label: 'Custom' },
      ],
    },
  ]

  const filteredProviders = useMemo(() => {
    if (filters.length === 0) return PROVIDERS_DATA

    return PROVIDERS_DATA.filter((provider) => {
      return filters.every((filter) => {
        const fieldValue = provider[filter.field as keyof Provider]
        const filterValues = filter.values

        switch (filter.operator) {
          case 'is':
            return fieldValue === filterValues[0]
          case 'isNot':
            return fieldValue !== filterValues[0]
          case 'contains':
            return String(fieldValue).toLowerCase().includes(String(filterValues[0]).toLowerCase())
          case 'notContains':
            return !String(fieldValue).toLowerCase().includes(String(filterValues[0]).toLowerCase())
          case 'startsWith':
            return String(fieldValue).toLowerCase().startsWith(String(filterValues[0]).toLowerCase())
          case 'endsWith':
            return String(fieldValue).toLowerCase().endsWith(String(filterValues[0]).toLowerCase())
          default:
            return true
        }
      })
    })
  }, [filters])

  return (
    <div className='flex flex-col gap-6'>
      {/* Stats */}
      <div className='grid grid-cols-1 md:grid-cols-3 gap-4'>
        <Card>
          <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
            <CardTitle className='text-sm font-medium'>Total Providers</CardTitle>
            <Database className='h-4 w-4 text-muted-foreground' />
          </CardHeader>
          <CardContent>
            <div className='text-2xl font-bold'>5</div>
            <p className='text-xs text-muted-foreground'>3 active</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
            <CardTitle className='text-sm font-medium'>Federated Users</CardTitle>
            <Users className='h-4 w-4 text-muted-foreground' />
          </CardHeader>
          <CardContent>
            <div className='text-2xl font-bold'>2,847</div>
            <p className='text-xs text-muted-foreground'>From all providers</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
            <CardTitle className='text-sm font-medium'>Success Rate</CardTitle>
            <CheckCircle className='h-4 w-4 text-muted-foreground' />
          </CardHeader>
          <CardContent>
            <div className='text-2xl font-bold'>98.5%</div>
            <p className='text-xs text-muted-foreground'>Last 7 days</p>
          </CardContent>
        </Card>
      </div>

      {/* Filters */}
      <Filters
        filters={filters}
        fields={filterFields}
        onChange={setFilters}
      />

      {/* Providers List */}
      <Card>
        <CardHeader>
          <div className='flex items-center justify-between'>
            <CardTitle>Federation Providers ({filteredProviders.length})</CardTitle>
            <Button variant='outline' size='sm'>
              <RefreshCw className='h-4 w-4 mr-2' />
              Sync All
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <div className='space-y-2'>
            {filteredProviders.map((provider, index) => {
              const Icon = PROVIDER_ICONS[provider.name as keyof typeof PROVIDER_ICONS]

              return (
                <div
                  key={index}
                  className='group flex items-center gap-4 p-4 border rounded-lg hover:bg-accent/50 transition-all cursor-pointer'
                >
                  {/* Icon */}
                  <div className='flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 shrink-0'>
                    <Icon className='h-5 w-5 text-primary' />
                  </div>

                  {/* Main Info */}
                  <div className='flex-1 min-w-0'>
                    <div className='flex items-center gap-2 mb-1'>
                      <h4 className='font-medium text-sm'>{provider.name}</h4>
                      <Badge variant='outline' className='text-xs'>
                        {provider.type}
                      </Badge>
                      <Badge variant='outline' className='text-xs'>
                        {provider.priority}
                      </Badge>
                    </div>
                    <div className='flex items-center gap-3 text-xs text-muted-foreground'>
                      <span className='flex items-center gap-1'>
                        <Server className='h-3 w-3' />
                        {provider.connection}
                      </span>
                      <span className='flex items-center gap-1'>
                        <Users className='h-3 w-3' />
                        {provider.users.toLocaleString()} users
                      </span>
                    </div>
                  </div>

                  {/* Status */}
                  <div className='flex items-center gap-3 shrink-0'>
                    <div className='flex items-center gap-2'>
                      <CircleDot className={`h-3 w-3 ${
                        provider.status === 'active' ? 'text-green-500 fill-green-500' :
                        provider.status === 'syncing' ? 'text-blue-500 fill-blue-500 animate-pulse' :
                        'text-gray-400 fill-gray-400'
                      }`} />
                      <span className='text-xs text-muted-foreground capitalize'>{provider.status}</span>
                    </div>
                    <div className='flex items-center gap-1 text-xs text-muted-foreground'>
                      <Clock className='h-3 w-3' />
                      {provider.lastSync}
                    </div>
                  </div>

                  {/* Actions */}
                  <div className='flex items-center gap-1 shrink-0'>
                    <Button
                      variant='ghost'
                      size='sm'
                      className='opacity-0 group-hover:opacity-100 transition-opacity'
                    >
                      <Settings className='h-4 w-4' />
                    </Button>
                    <ChevronRight className='h-4 w-4 text-muted-foreground' />
                  </div>
                </div>
              )
            })}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
