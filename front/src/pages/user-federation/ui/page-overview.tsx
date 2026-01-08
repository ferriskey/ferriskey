import { Button } from '@/components/ui/button'
import { Filters, Filter, FilterFieldsConfig } from '@/components/ui/filters'
import { useState, useMemo } from 'react'
import StatisticsCard from '../components/statistics-card'
import ProviderCard from '../components/provider-card'
import EmptyStateProviders from '../components/empty-state-providers'

import {
  Database,
  Users,
  CheckCircle,
  Building2,
  Server,
  Wrench,
  Key,
  RefreshCw,
  Plus,
} from 'lucide-react'
import BlockContent from '@/components/ui/block-content'

const PROVIDER_ICONS = {
  'Corporate LDAP': Building2,
  'Active Directory': Server,
  'Development LDAP': Wrench,
  'Legacy Kerberos': Key,
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

interface PageOverviewProps {
  onCreateProvider: (type?: 'LDAP' | 'Kerberos') => void
  providers?: Provider[]
  isLoading?: boolean
}

export default function PageOverview({ onCreateProvider, providers = [], isLoading }: PageOverviewProps) {
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
    if (filters.length === 0) return providers

    return providers.filter((provider) => {
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
  }, [filters, providers])

  if (isLoading) {
    return <div className='p-4 text-center'>Loading providers...</div>
  }

  if (providers.length === 0) {
    return (
      <EmptyStateProviders onCreateProvider={onCreateProvider} />
    )
  }

  return (
    <div className='flex flex-col gap-6'>
      {/* Stats */}
      <div className='grid grid-cols-1 md:grid-cols-3 gap-4'>
        <StatisticsCard
          title='Total Providers'
          value={filteredProviders.length}
          description={`${filteredProviders.filter(p => p.status === 'active').length} active`}
          icon={Database}
        />
        <StatisticsCard
          title='Federated Users'
          value={filteredProviders.reduce((acc, p) => acc + p.users, 0)}
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
        <Button onClick={() => onCreateProvider('LDAP')}>
          <Plus className='h-4 w-4 mr-2' />
          Add Provider
        </Button>
      </div>

      {/* Providers List */}
      <BlockContent title={`Federation Providers (${(filteredProviders.length)})`} headHeight='h-12' headRight={<Button variant='outline' size='sm'>
        <RefreshCw className='h-4 w-4 mr-2' />
        Sync All
      </Button>}>
        <div className='space-y-2'>
          {filteredProviders.map((provider, index) => {
            const Icon = PROVIDER_ICONS[provider.name as keyof typeof PROVIDER_ICONS] || Database

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
      </BlockContent>
    </div>
  )
}
