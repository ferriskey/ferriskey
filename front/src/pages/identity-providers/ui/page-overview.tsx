import { DataTable } from '@/components/ui/data-table'
import { Edit, ExternalLink, Trash2, ScanFace, CheckCircle, XCircle, Layers } from 'lucide-react'
import { columns } from '../columns/list-provider.column'
import type { IdentityProviderListItem } from '../types'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import StatisticsCard from '../components/statistics-card'
import ProvidersEmptyState from '../components/providers-empty-state'

interface Statistics {
  totalProviders: number
  enabledProviders: number
  disabledProviders: number
  providerTypes: number
}

interface ConfirmState {
  title: string
  description: string
  open: boolean
  onConfirm: () => void
}

export interface PageOverviewProps {
  isLoading?: boolean
  data: IdentityProviderListItem[]
  realmName: string
  statistics: Statistics
  filters: Filter[]
  filterFields: FilterFieldsConfig
  onFiltersChange: (filters: Filter[]) => void
  confirm: ConfirmState
  onConfirmClose: () => void
  handleDeleteSelected: (items: IdentityProviderListItem[]) => void
  handleClickRow: (providerId: string) => void
  handleCreateProvider: () => void
  onRowDelete: (provider: IdentityProviderListItem) => void
}

export default function PageOverview({
  data,
  isLoading,
  statistics,
  filters,
  filterFields,
  onFiltersChange,
  confirm,
  onConfirmClose,
  handleDeleteSelected,
  handleClickRow,
  handleCreateProvider,
  onRowDelete,
}: PageOverviewProps) {
  const { totalProviders, enabledProviders, disabledProviders, providerTypes } = statistics
  const isEmpty = !isLoading && data.length === 0

  // Show empty state when there are no providers
  if (isEmpty) {
    return (
      <div className='flex flex-col gap-6'>
        <ProvidersEmptyState onCreateProvider={handleCreateProvider} />
      </div>
    )
  }

  return (
    <div className='flex flex-col gap-6'>
      {/* Statistics Cards */}
      <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-4'>
        <StatisticsCard
          title='Total Providers'
          value={totalProviders}
          description='All configured providers'
          icon={ScanFace}
          isLoading={isLoading}
        />

        <StatisticsCard
          title='Enabled Providers'
          value={enabledProviders}
          description={
            enabledProviders > 0 && totalProviders > 0 ? (
              <span className='text-emerald-600 font-medium'>
                {((enabledProviders / totalProviders) * 100).toFixed(0)}% active
              </span>
            ) : (
              'No enabled providers'
            )
          }
          icon={CheckCircle}
          isLoading={isLoading}
        />

        <StatisticsCard
          title='Disabled Providers'
          value={disabledProviders}
          description='Inactive providers'
          icon={XCircle}
          isLoading={isLoading}
        />

        <StatisticsCard
          title='Provider Types'
          value={providerTypes}
          description='Different protocols configured'
          icon={Layers}
          isLoading={isLoading}
        />
      </div>

      {/* Data Table */}
      <DataTable
        data={data}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder='Search providers by name or alias...'
        searchKeys={['display_name', 'alias']}
        enableSelection={true}
        enableFilters={true}
        filterFields={filterFields}
        filters={filters}
        onFiltersChange={onFiltersChange}
        onRowClick={(provider) => {
          handleClickRow(provider.id)
        }}
        onDeleteSelected={handleDeleteSelected}
        createData={{
          label: 'Create Provider',
          onClick: handleCreateProvider,
        }}
        rowActions={[
          {
            label: 'Edit',
            icon: <Edit className='h-4 w-4' />,
            onClick: (provider) => handleClickRow(provider.id),
          },
          {
            label: 'View',
            icon: <ExternalLink className='h-4 w-4' />,
            onClick: (provider) => handleClickRow(provider.id),
          },
          {
            label: 'Delete',
            icon: <Trash2 className='h-4 w-4' />,
            variant: 'destructive',
            onClick: (provider) => onRowDelete(provider),
          },
        ]}
      />

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={onConfirmClose}
      />
    </div>
  )
}
