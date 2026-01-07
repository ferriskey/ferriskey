import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Filters, Filter, FilterFieldsConfig } from '@/components/ui/filters'
import { useState, useMemo } from 'react'
import StatisticsCard from '../components/statistics-card'
import ProviderCard from '../components/provider-card'

import {
  Database,
  Users,
  CheckCircle,
  Building2,
  Server,
  Wrench,
  Key,
  Cog,
  RefreshCw,
  Plus,
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
  status: 'active' | 'syncing' | 'inactive'
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

interface PageOverviewProps {
  onCreateProvider: () => void
}

export default function PageOverview({ onCreateProvider }: PageOverviewProps) {
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
        <StatisticsCard
          title='Total Providers'
          value={5}
          description='3 active'
          icon={Database}
        />
        <StatisticsCard
          title='Federated Users'
          value={2847}
          description='From all providers'
          icon={Users}
        />
        <StatisticsCard
          title='Success Rate'
          value={98.5}
          description='Last 7 days'
          icon={CheckCircle}
        />
      </div>

      {/* Filters and Actions */}
      <div className='flex items-center justify-between gap-4'>
        <Filters
          filters={filters}
          fields={filterFields}
          onChange={setFilters}
        />
        <Button onClick={onCreateProvider}>
          <Plus className='h-4 w-4 mr-2' />
          Add Provider
        </Button>
      </div>

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
                <ProviderCard
                  key={index}
                  name={provider.name}
                  type={provider.type}
                  status={provider.status}
                  users={provider.users}
                  lastSync={provider.lastSync}
                  connection={provider.connection}
                  priority={provider.priority}
                  icon={Icon}
                  onClick={() => console.log('Provider clicked:', provider.name)}
                  onSettings={() => console.log('Settings clicked:', provider.name)}
                />
              )
            })}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
