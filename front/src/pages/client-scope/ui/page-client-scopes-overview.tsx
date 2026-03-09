import { Schemas } from '@/api/api.client'
import { EntityAvatar } from '@/components/ui/entity-avatar'
import { OverviewList } from '@/components/ui/overview-list'
import StatisticsCard from '../components/statistics-card'

import ClientScope = Schemas.ClientScope
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Trash2 } from 'lucide-react'
import { useState } from 'react'

interface Statistics {
  totalScopes: number
  defaultScopes: number
  optionalScopes: number
  withProtocolMappers: number
}

export interface PageClientScopesOverviewProps {
  isLoading?: boolean
  data: ClientScope[]
  statistics: Statistics
  handleClickRow: (scopeId: string) => void
  handleDeleteScope: (scopeId: string) => void
  isDeleting?: boolean
}

function ScopeTypeBadge({ scopeType }: { scopeType: string }) {
  return scopeType === 'DEFAULT' ? (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-blue-300 text-blue-500 text-xs font-mono bg-blue-50 dark:bg-blue-500/10 dark:border-blue-400/40'>
      default
    </span>
  ) : (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-purple-300 text-purple-500 text-xs font-mono bg-purple-50 dark:bg-purple-500/10 dark:border-purple-400/40'>
      optional
    </span>
  )
}

function ProtocolBadge({ protocol }: { protocol: string }) {
  return (
    <span className='inline-flex items-center px-3 py-1 rounded-md text-xs font-semibold border border-border text-muted-foreground bg-muted/50'>
      {protocol}
    </span>
  )
}

export default function PageClientScopesOverview({
  data,
  isLoading,
  statistics,
  handleClickRow,
  handleDeleteScope,
  isDeleting,
}: PageClientScopesOverviewProps) {
  const { totalScopes, defaultScopes, optionalScopes, withProtocolMappers } = statistics
  const [pendingDelete, setPendingDelete] = useState<ClientScope | null>(null)

  const handleConfirmDelete = () => {
    if (!pendingDelete) return
    handleDeleteScope(pendingDelete.id)
    setPendingDelete(null)
  }

  return (
    <div className='flex flex-col gap-6'>
      <div>
        <p className='text-xs text-muted-foreground mb-3'>Client scope overview</p>
        <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-4'>
          <StatisticsCard
            title='Total scopes'
            value={totalScopes}
            description='All registered client scopes'
            isLoading={isLoading}
          />
          <StatisticsCard
            title='Default scopes'
            value={defaultScopes}
            description='Automatically assigned scopes'
            isLoading={isLoading}
          />
          <StatisticsCard
            title='Optional scopes'
            value={optionalScopes}
            description='Non-default client scopes'
            isLoading={isLoading}
          />
          <StatisticsCard
            title='With mappers'
            value={withProtocolMappers}
            description='Scopes containing protocol mappers'
            isLoading={isLoading}
          />
        </div>
      </div>

      <OverviewList
        data={data}
        isLoading={isLoading}
        searchKeys={['name', 'description', 'protocol']}
        searchPlaceholder='Search client scopes...'
        title={(n) => `Client Scopes (${n})`}
        emptyLabel='No client scopes found.'
        renderRow={(scope) => (
          <div className='flex items-center w-full'>
            <Button
              type='button'
              variant='ghost'
              onClick={() => handleClickRow(scope.id)}
              className='flex items-center justify-between px-8 py-4 hover:bg-muted/40 transition-colors cursor-pointer flex-1 h-full'
            >
              <div className='flex items-center gap-4'>
                <EntityAvatar label={scope.name} color='#0EA5E9' />
                <div>
                  <div className='flex items-center gap-2.5'>
                    <span className='text-base font-medium'>{scope.name}</span>
                    <ScopeTypeBadge scopeType={scope.default_scope_type} />
                  </div>
                  <div className='text-sm text-muted-foreground mt-0.5'>
                    {scope.description || `scope_id: ${scope.id}`}
                  </div>
                </div>
              </div>
              <ProtocolBadge protocol={scope.protocol} />
            </Button>
            <div className='pr-4'>
              <Button
                type='button'
                variant='ghost'
                size='sm'
                onClick={(e) => {
                  e.stopPropagation()
                  setPendingDelete(scope)
                }}
                className='text-muted-foreground hover:text-destructive hover:bg-destructive/10'
              >
                <Trash2 className='h-4 w-4' />
              </Button>
            </div>
          </div>
        )}
      />

      <Dialog open={!!pendingDelete} onOpenChange={(open) => !open && setPendingDelete(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Client Scope</DialogTitle>
          </DialogHeader>
          <DialogBody>
            <p className='text-sm text-muted-foreground'>
              Are you sure you want to delete{' '}
              <span className='font-semibold text-foreground'>{pendingDelete?.name}</span>? This
              action is irreversible and will remove all associated protocol mappers and client
              mappings.
            </p>
          </DialogBody>
          <DialogFooter>
            <Button variant='outline' onClick={() => setPendingDelete(null)} disabled={isDeleting}>
              Cancel
            </Button>
            <Button variant='destructive' onClick={handleConfirmDelete} disabled={isDeleting}>
              {isDeleting ? 'Deleting...' : 'Delete'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
