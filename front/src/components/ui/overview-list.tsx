import { ReactNode, useState, useMemo } from 'react'
import { Plus, Search } from 'lucide-react'
import { Input } from './input'
import { Button } from './button'
import { Skeleton } from './skeleton'
import { cn } from '@/lib/utils'

export interface OverviewListProps<T extends { id: string }> {
  data: T[]
  isLoading?: boolean
  searchKeys?: (keyof T)[]
  searchPlaceholder?: string
  pageSize?: number
  title?: (count: number) => string
  renderRow: (item: T) => ReactNode
  emptyLabel?: string
  action?: {
    label: string
    onClick: () => void
  }
}

export function OverviewList<T extends { id: string }>({
  data,
  isLoading = false,
  searchKeys = [],
  searchPlaceholder = 'Search...',
  pageSize = 10,
  title = (n) => `Items (${n})`,
  renderRow,
  emptyLabel = 'No items found.',
  action,
}: OverviewListProps<T>) {
  const [search, setSearch] = useState('')
  const [currentPage, setCurrentPage] = useState(1)

  const filteredData = useMemo(() => {
    if (!search.trim() || searchKeys.length === 0) return data
    const q = search.toLowerCase()
    return data.filter((item) =>
      searchKeys.some((key) => String(item[key] ?? '').toLowerCase().includes(q))
    )
  }, [data, search, searchKeys])

  const totalPages = Math.ceil(filteredData.length / pageSize)
  const paginatedData = useMemo(() => {
    const start = (currentPage - 1) * pageSize
    return filteredData.slice(start, start + pageSize)
  }, [filteredData, currentPage, pageSize])

  const rangeStart = filteredData.length === 0 ? 0 : (currentPage - 1) * pageSize + 1
  const rangeEnd = Math.min(currentPage * pageSize, filteredData.length)

  const handleSearch = (value: string) => {
    setSearch(value)
    setCurrentPage(1)
  }

  return (
    <div>
      {/* Header */}
      <div className='flex items-center justify-between mb-3'>
        <h2 className='text-base font-semibold'>{title(filteredData.length)}</h2>
        <div className='flex items-center gap-2'>
          {searchKeys.length > 0 && (
            <div className='relative'>
              <Search className='absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground' />
              <Input
                type='search'
                placeholder={searchPlaceholder}
                className='pl-9 h-9 w-64 bg-background text-sm'
                value={search}
                onChange={(e) => handleSearch(e.target.value)}
              />
            </div>
          )}
          {action && (
            <Button size='sm' onClick={action.onClick}>
              <Plus className='h-4 w-4' />
              {action.label}
            </Button>
          )}
        </div>
      </div>

      {/* List body */}
      <div className='-mx-8 border-t border-b overflow-hidden'>
        {isLoading ? (
          Array.from({ length: 6 }).map((_, i) => (
            <div key={i} className={cn('flex items-center justify-between px-8 py-4', i < 5 ? 'border-b' : '')}>
              <div className='flex items-center gap-3'>
                <Skeleton className='h-10 w-10 rounded-md' />
                <div className='space-y-2'>
                  <Skeleton className='h-4 w-40' />
                  <Skeleton className='h-3 w-32' />
                </div>
              </div>
              <Skeleton className='h-6 w-16 rounded-md' />
            </div>
          ))
        ) : paginatedData.length === 0 ? (
          <div className='flex items-center justify-center h-24 text-sm text-muted-foreground'>
            {emptyLabel}
          </div>
        ) : (
          paginatedData.map((item, index) => (
            <div key={item.id} className={index < paginatedData.length - 1 ? 'border-b' : ''}>{renderRow(item)}</div>
          ))
        )}
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className='flex items-center justify-between mt-4 px-1'>
          <span className='text-sm text-muted-foreground'>
            {rangeStart}-{rangeEnd} sur {filteredData.length}
          </span>
          <div className='flex items-center gap-1'>
            <Button
              variant='outline'
              size='sm'
              onClick={() => setCurrentPage((p) => Math.max(p - 1, 1))}
              disabled={currentPage <= 1}
              className='h-8'
            >
              Precedent
            </Button>
            {Array.from({ length: totalPages }, (_, i) => i + 1).map((page) => (
              <Button
                key={page}
                variant={page === currentPage ? 'default' : 'outline'}
                size='sm'
                onClick={() => setCurrentPage(page)}
                className='h-8 w-8 p-0'
              >
                {page}
              </Button>
            ))}
            <Button
              variant='outline'
              size='sm'
              onClick={() => setCurrentPage((p) => Math.min(p + 1, totalPages))}
              disabled={currentPage >= totalPages}
              className='h-8'
            >
              Suivant
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}
