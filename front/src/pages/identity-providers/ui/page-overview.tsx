import { Trash2, ScanFace, CheckCircle, XCircle, Layers } from 'lucide-react'
import type { IdentityProviderListItem } from '../types'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import StatisticsCard from '../components/statistics-card'
import ProvidersEmptyState from '../components/providers-empty-state'
import { OverviewList } from '@/components/ui/overview-list'
import { EntityAvatar } from '@/components/ui/entity-avatar'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

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
  confirm: ConfirmState
  onConfirmClose: () => void
  handleDeleteSelected: (items: IdentityProviderListItem[]) => void
  handleClickRow: (providerId: string) => void
  handleCreateProvider: () => void
  onRowDelete: (provider: IdentityProviderListItem) => void
}

const PROVIDER_COLORS: Record<string, string> = {
  oidc: '#6366F1',
  oauth2: '#F59E0B',
  saml: '#10B981',
  ldap: '#3B82F6',
}

function getProviderColor(providerType: string): string {
  return PROVIDER_COLORS[providerType?.toLowerCase()] ?? '#8B5CF6'
}

export default function PageOverview({
  data,
  isLoading,
  statistics,
  confirm,
  onConfirmClose,
  handleClickRow,
  handleCreateProvider,
  onRowDelete,
}: PageOverviewProps) {
  const { totalProviders, enabledProviders, disabledProviders, providerTypes } = statistics
  const isEmpty = !isLoading && data.length === 0

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

      {/* Providers List */}
      <OverviewList
        data={data}
        searchKeys={['display_name', 'alias']}
        searchPlaceholder='Search providers...'
        title={(n) => `Identity Providers (${n})`}
        emptyLabel='No identity providers configured.'
        renderRow={(provider) => (
          <div
            className='flex items-center justify-between px-8 py-4 hover:bg-muted/40 cursor-pointer transition-colors'
            onClick={() => handleClickRow(provider.id)}
          >
            <div className='flex items-center gap-4'>
              <EntityAvatar
                label={provider.display_name || provider.alias}
                color={getProviderColor(provider.provider_id)}
              />
              <div>
                <div className='flex items-center gap-2'>
                  <span className='text-base font-medium'>{provider.display_name || provider.alias}</span>
                  <span className='inline-flex items-center px-2 py-0.5 rounded text-xs font-mono border border-border text-muted-foreground bg-muted/50'>
                    {provider.provider_id}
                  </span>
                  <span
                    className={cn(
                      'inline-flex items-center px-2 py-0.5 rounded text-xs font-mono border',
                      provider.enabled
                        ? 'border-green-300 text-green-600 bg-green-50 dark:bg-green-500/10 dark:border-green-400/40'
                        : 'border-border text-muted-foreground bg-muted/50'
                    )}
                  >
                    {provider.enabled ? 'enabled' : 'disabled'}
                  </span>
                </div>
                <div className='text-sm text-muted-foreground mt-0.5 font-mono'>{provider.alias}</div>
              </div>
            </div>
            <Button
              variant='ghost'
              size='icon'
              className='text-muted-foreground hover:text-destructive'
              onClick={(e) => {
                e.stopPropagation()
                onRowDelete(provider)
              }}
            >
              <Trash2 className='h-4 w-4' />
            </Button>
          </div>
        )}
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
