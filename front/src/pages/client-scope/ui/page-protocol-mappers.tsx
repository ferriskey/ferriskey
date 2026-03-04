import { Fragment } from 'react/jsx-runtime'
import { Schemas } from '@/api/api.client'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { OverviewList } from '@/components/ui/overview-list'
import { EntityAvatar } from '@/components/ui/entity-avatar'
import { Button } from '@/components/ui/button'
import { Trash2 } from 'lucide-react'
import StatisticsCard from '../components/statistics-card'

import ProtocolMapper = Schemas.ProtocolMapper

// ─── Badges ──────────────────────────────────────────────────────────────────

function MapperCategoryBadge({ mapperType }: { mapperType: string }) {
  const t = mapperType.toLowerCase()

  if (t.includes('role')) {
    return (
      <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-purple-300 text-purple-500 text-xs font-mono bg-purple-50 dark:bg-purple-500/10 dark:border-purple-400/40'>
        role
      </span>
    )
  }
  if (t.includes('audience')) {
    return (
      <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-emerald-300 text-emerald-500 text-xs font-mono bg-emerald-50 dark:bg-emerald-500/10 dark:border-emerald-400/40'>
        audience
      </span>
    )
  }
  if (t.includes('hardcoded')) {
    return (
      <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-orange-300 text-orange-500 text-xs font-mono bg-orange-50 dark:bg-orange-500/10 dark:border-orange-400/40'>
        hardcoded
      </span>
    )
  }
  if (t.includes('attribute')) {
    return (
      <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-blue-300 text-blue-500 text-xs font-mono bg-blue-50 dark:bg-blue-500/10 dark:border-blue-400/40'>
        attribute
      </span>
    )
  }
  if (t.includes('property') || t.includes('full-name') || t.includes('usermodel')) {
    return (
      <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-blue-300 text-blue-500 text-xs font-mono bg-blue-50 dark:bg-blue-500/10 dark:border-blue-400/40'>
        identity
      </span>
    )
  }
  return (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-border text-muted-foreground text-xs font-mono bg-muted/50'>
      custom
    </span>
  )
}

// ─── Types ────────────────────────────────────────────────────────────────────

interface ConfirmState {
  title: string
  description: string
  open: boolean
  onConfirm: () => void
}

interface Statistics {
  total: number
  roleMappers: number
  identityMappers: number
}

export interface PageProtocolMappersProps {
  mappers: ProtocolMapper[]
  isLoading: boolean
  statistics: Statistics
  confirm: ConfirmState
  onConfirmClose: () => void
  onAdd: () => void
  onClickRow: (mapper: ProtocolMapper) => void
  onDelete: (mapper: ProtocolMapper) => void
}

// ─── Component ────────────────────────────────────────────────────────────────

export default function PageProtocolMappers({
  mappers,
  isLoading,
  statistics,
  confirm,
  onConfirmClose,
  onAdd,
  onClickRow,
  onDelete,
}: PageProtocolMappersProps) {
  const { total, roleMappers, identityMappers } = statistics

  return (
    <Fragment>
      <div className='flex flex-col gap-6'>
        {/* Statistics Cards */}
        <div>
          <p className='text-xs text-muted-foreground mb-3'>Mapper overview</p>
          <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-3'>
            <StatisticsCard
              title='Total mappers'
              value={total}
              description='All configured protocol mappers'
              isLoading={isLoading}
            />
            <StatisticsCard
              title='Role mappers'
              value={roleMappers}
              description={
                roleMappers > 0 && total > 0 ? (
                  <span className='text-purple-600 font-medium'>
                    {((roleMappers / total) * 100).toFixed(0)}% of total
                  </span>
                ) : (
                  'No role mappers'
                )
              }
              isLoading={isLoading}
            />
            <StatisticsCard
              title='Identity mappers'
              value={identityMappers}
              description={
                identityMappers > 0 && total > 0 ? (
                  <span className='text-blue-600 font-medium'>
                    {((identityMappers / total) * 100).toFixed(0)}% of total
                  </span>
                ) : (
                  'No identity mappers'
                )
              }
              isLoading={isLoading}
            />
          </div>
        </div>

        {/* Mapper List */}
        <OverviewList
          data={mappers}
          isLoading={isLoading}
          searchKeys={['name', 'mapper_type']}
          searchPlaceholder='Search protocol mappers...'
          title={(n) => `Protocol Mappers (${n})`}
          emptyLabel='No protocol mappers configured.'
          action={{ label: 'Add Mapper', onClick: onAdd }}
          renderRow={(mapper) => (
            <div
              onClick={() => onClickRow(mapper)}
              className='flex items-center justify-between px-8 py-4 border-b last:border-b-0 hover:bg-muted/40 cursor-pointer transition-colors'
            >
              <div className='flex items-center gap-4'>
                <EntityAvatar label={mapper.name} color='#6366F1' />
                <div>
                  <div className='flex items-center gap-2.5'>
                    <span className='text-base font-medium'>{mapper.name}</span>
                    <MapperCategoryBadge mapperType={mapper.mapper_type} />
                  </div>
                  <div className='text-sm text-muted-foreground mt-0.5 font-mono'>
                    {mapper.mapper_type}
                  </div>
                </div>
              </div>
              <Button
                variant='ghost'
                size='icon'
                className='h-8 w-8 shrink-0 text-muted-foreground hover:text-destructive hover:bg-destructive/10'
                onClick={(e) => {
                  e.stopPropagation()
                  onDelete(mapper)
                }}
              >
                <Trash2 className='h-4 w-4' />
              </Button>
            </div>
          )}
        />
      </div>

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={onConfirmClose}
      />
    </Fragment>
  )
}
