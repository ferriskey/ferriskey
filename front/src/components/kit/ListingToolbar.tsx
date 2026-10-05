import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { X } from 'lucide-react'
import { Button } from './button'
import { SearchInput } from './SearchInput'
import { clearColumnFilters, countActiveFilters, type ColumnFilterField } from './column-filter-state'
import type { PagedListing } from './use-paged-listing'
import { cn } from '@/lib/utils'

const SEARCH_KEY = 'search'

export function ListingToolbar({
  listing,
  columns,
  search,
  leading,
  trailing,
  className,
}: {
  listing: PagedListing
  columns: { filters?: ColumnFilterField[] }[]
  search?: { placeholder: string }
  leading?: ReactNode
  trailing?: ReactNode
  className?: string
}) {
  const { t } = useTranslation()
  const activeFilters = countActiveFilters(columns, listing.state.filters)

  if (!search && activeFilters === 0 && !leading && !trailing) return null

  const clear = () =>
    listing.setFilters(clearColumnFilters(columns.flatMap((column) => column.filters ?? [])))

  return (
    <div className={cn('flex flex-wrap items-center gap-2', className)}>
      {leading}
      {search && (
        <SearchInput
          value={listing.drafts[SEARCH_KEY] ?? ''}
          onChange={(value) => listing.setDraft(SEARCH_KEY, value)}
          placeholder={search.placeholder}
        />
      )}
      {activeFilters > 0 && (
        <Button variant='ghost' size='sm' onClick={clear}>
          <X />
          {t('common:listing.clear_filters', { count: activeFilters })}
        </Button>
      )}
      {trailing}
    </div>
  )
}
