import { cn } from '@/lib/utils'
import { ChevronLeft, ChevronRight, MoreVertical, Search, Trash2 } from 'lucide-react'
import { AnimatePresence, motion } from 'motion/react'
import { ReactNode, useMemo, useState } from 'react'
import { Button } from './button'
import { Checkbox } from './checkbox'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from './dropdown-menu'
import { Input } from './input'
import { Skeleton } from './skeleton'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from './table'
import { Filters, Filter, FilterFieldsConfig } from './filters'

export type ColumnDef<T> = {
  id: string
  header: string
  accessorKey?: keyof T
  cell?: (item: T) => ReactNode
  enableSorting?: boolean
  enableFilter?: boolean
}

export type RowAction<T> = {
  label: string
  icon?: ReactNode
  onClick: (item: T) => void
  variant?: 'default' | 'destructive'
}

export type DataTableProps<T> = {
  data: T[]
  columns: ColumnDef<T>[]
  title?: string
  description?: string
  isLoading?: boolean
  searchPlaceholder?: string
  searchKeys?: (keyof T)[]
  rowActions?: RowAction<T>[]
  enablePagination?: boolean
  enableSelection?: boolean
  createData?: {
    label: string
    onClick: () => void
  }
  emptyState?: React.ReactNode
  onSelectionChange?: (selectedItems: T[]) => void
  pageSize?: number
  onDeleteSelected?: (items: T[]) => void
  onRowClick?: (item: T) => void
  enableFilters?: boolean
  filterFields?: FilterFieldsConfig
  filters?: Filter[]
  onFiltersChange?: (filters: Filter[]) => void
}

export function DataTable<T extends { id: string }>({
  data,
  columns,
  title,
  description,
  isLoading = false,
  searchPlaceholder = 'Rechercher...',
  searchKeys = [],
  rowActions = [],
  enablePagination = true,
  enableSelection = false,
  createData,
  emptyState,
  onSelectionChange,
  pageSize = 10,
  onDeleteSelected,
  onRowClick,
  enableFilters = false,
  filterFields = [],
  filters = [],
  onFiltersChange,
}: DataTableProps<T>) {
  const [search, setSearch] = useState('')
  const [selectedItems, setSelectedItems] = useState<T[]>([])
  const [currentPage, setCurrentPage] = useState(1)
  const [isConfirmingDelete, setIsConfirmingDelete] = useState(false)

  const filteredData = useMemo(() => {
    if (!search || search.length === 0 || searchKeys.length === 0) return data

    return data.filter((item) => {
      return searchKeys.some((key) => {
        const value = item[key]
        return value && String(value).toLowerCase().includes(search.toLowerCase())
      })
    })
  }, [data, search, searchKeys])

  const totalPages = Math.ceil(filteredData.length / pageSize)
  const paginatedData = useMemo(() => {
    if (!enablePagination) return filteredData

    const start = (currentPage - 1) * pageSize
    return filteredData.slice(start, start + pageSize)
  }, [filteredData, currentPage, pageSize, enablePagination])

  const handleSelectItem = (item: T) => {
    const isSelected = selectedItems.some((i) => i.id === item.id)

    const newSelectedItems: T[] = isSelected
      ? selectedItems.filter((i) => i.id !== item.id)
      : [...selectedItems, item]

    setSelectedItems(newSelectedItems)

    if (onSelectionChange) {
      onSelectionChange(newSelectedItems)
    }
  }

  const handleSelectAll = (checked: boolean) => {
    if (checked) {
      setSelectedItems(paginatedData)
    } else {
      setSelectedItems([])
    }
    if (onSelectionChange) {
      onSelectionChange(checked ? paginatedData : [])
    }
  }

  const handleConfirmDelete = () => {
    if (onDeleteSelected) {
      onDeleteSelected(selectedItems)
      setSelectedItems([])
      setIsConfirmingDelete(false)
    }
  }

  return (
    <div className='space-y-6 flex flex-col'>
      {/* Header Section */}
      <div className='flex flex-col gap-4'>
        {/* Title and Description */}
        {(title || description) && (
          <div className='flex flex-col gap-1'>
            {title && <h2 className='text-2xl font-semibold tracking-tight'>{title}</h2>}
            {description && <p className='text-sm text-muted-foreground'>{description}</p>}
          </div>
        )}

        {/* Search Bar and Create Button */}
        <div className='flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4'>
          <div className='flex flex-1 items-center gap-3 w-full sm:w-auto'>
            {searchKeys.length > 0 && (
              <div className='relative flex-1 sm:flex-initial sm:min-w-[320px]'>
                <Search className='absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground' />
                <Input
                  type='search'
                  placeholder={searchPlaceholder}
                  className='pl-10 h-10 bg-background'
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                />
              </div>
            )}
          </div>

          {createData && (
            <Button variant='default' onClick={createData.onClick} className='w-full sm:w-auto'>
              {createData.label}
            </Button>
          )}
        </div>

        {/* Filters Section */}
        {enableFilters && filterFields && filterFields.length > 0 && (
          <div className='flex items-center gap-2'>
            <Filters
              fields={filterFields}
              filters={filters}
              onChange={onFiltersChange || (() => { })}
              variant='outline'
              size='sm'
            />
          </div>
        )}
      </div>

      {/* Table Section */}
      <div className='rounded-lg border bg-card shadow-sm'>
        {isLoading ? (
          <TableSkeleton columns={columns.length} enableSelection={enableSelection} />
        ) : (
          <div className='relative w-full overflow-auto'>
            <Table>
              <TableHeader>
                <TableRow>
                  {enableSelection && (
                    <TableHead className='w-[40px] px-2'>
                      <Checkbox
                        checked={
                          paginatedData.length > 0 &&
                          selectedItems.length === paginatedData.length
                        }
                        onCheckedChange={handleSelectAll}
                        aria-label='Select all'
                      />
                    </TableHead>
                  )}

                  {columns.map((column) => (
                    <TableHead key={column.id}>{column.header}</TableHead>
                  ))}

                  {rowActions.length > 0 && <TableHead className='w-[80px]'></TableHead>}
                </TableRow>
              </TableHeader>

              <TableBody>
                {paginatedData.length > 0 ? (
                  paginatedData.map((row) => (
                    <TableRow
                      key={row.id}
                      className={cn('hover:bg-muted/50', onRowClick && 'cursor-pointer')}
                    >
                      {enableSelection && (
                        <TableCell className='px-2'>
                          <Checkbox
                            checked={selectedItems.some((item) => item.id === row.id)}
                            onCheckedChange={() => handleSelectItem(row)}
                            aria-label={'Select row'}
                          />
                        </TableCell>
                      )}

                      {columns.map((column) => (
                        <TableCell
                          key={column.id}
                          onClick={() => {
                            if (onRowClick) {
                              onRowClick(row)
                            }
                          }}
                        >
                          {column.cell
                            ? column.cell(row)
                            : column.accessorKey
                              ? String(row[column.accessorKey] || '')
                              : ''}
                        </TableCell>
                      ))}

                      {rowActions.length > 0 && (
                        <TableCell className='text-right'>
                          <DropdownMenu modal={false}>
                            <DropdownMenuTrigger asChild>
                              <Button
                                variant='ghost'
                                size='icon'
                                className='h-8 w-8 cursor-pointer'
                              >
                                <span className='sr-only'>Ouvrir le menu</span>
                                <MoreVertical className='h-4 w-4' />
                              </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align='end'>
                              <DropdownMenuLabel>Actions</DropdownMenuLabel>
                              <DropdownMenuSeparator />
                              {rowActions.map((action, index) => (
                                <DropdownMenuItem
                                  key={index}
                                  onClick={() => action.onClick(row)}
                                  className={cn(
                                    'flex items-center gap-2 cursor-pointer',
                                    action.variant === 'destructive' && 'text-destructive'
                                  )}
                                >
                                  {action.icon}
                                  <span>{action.label}</span>
                                </DropdownMenuItem>
                              ))}
                            </DropdownMenuContent>
                          </DropdownMenu>
                        </TableCell>
                      )}
                    </TableRow>
                  ))
                ) : (
                  <TableRow>
                    <TableCell
                      colSpan={
                        columns.length +
                        (enableSelection ? 1 : 0) +
                        (rowActions.length > 0 ? 1 : 0)
                      }
                      className='h-24 text-center'
                    >
                      {emptyState || (search ? 'No result found' : 'No data available.')}
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </div>
        )}
      </div>

      {/* Pagination */}
      {enablePagination && totalPages > 1 && (
        <div className='flex flex-col sm:flex-row items-center justify-between gap-4 px-2 py-4 border-t bg-muted/30'>
          <div className='flex items-center gap-2'>
            <p className='text-sm text-muted-foreground'>
              Showing <span className='font-medium text-foreground'>{(currentPage - 1) * pageSize + 1}</span> to{' '}
              <span className='font-medium text-foreground'>
                {Math.min(currentPage * pageSize, filteredData.length)}
              </span>{' '}
              of <span className='font-medium text-foreground'>{filteredData.length}</span> results
            </p>
          </div>
          <div className='flex items-center gap-2'>
            <p className='text-sm text-muted-foreground'>
              Page <span className='font-medium text-foreground'>{currentPage}</span> of{' '}
              <span className='font-medium text-foreground'>{totalPages}</span>
            </p>
            <div className='flex items-center gap-1'>
              <Button
                variant='outline'
                size='sm'
                onClick={() => setCurrentPage((prev) => Math.max(prev - 1, 1))}
                disabled={currentPage <= 1}
                className='h-8 w-8 p-0'
              >
                <ChevronLeft className='h-4 w-4' />
                <span className='sr-only'>Previous page</span>
              </Button>
              <Button
                variant='outline'
                size='sm'
                onClick={() => setCurrentPage((prev) => Math.min(prev + 1, totalPages))}
                disabled={currentPage >= totalPages}
                className='h-8 w-8 p-0'
              >
                <ChevronRight className='h-4 w-4' />
                <span className='sr-only'>Next page</span>
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Bannière flottante de confirmation de suppression (style Discord) */}
      <AnimatePresence>
        {selectedItems.length > 0 && onDeleteSelected && (
          <motion.div
            initial={{ y: 100, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            exit={{ y: 100, opacity: 0 }}
            transition={{ type: 'spring', stiffness: 300, damping: 30 }}
            className='fixed bottom-6 left-1/2 transform -translate-x-1/2 z-50 w-full max-w-md bg-background shadow-2xl rounded-xl border-2 px-4 py-4'
          >
            <div className='flex items-center justify-between gap-4'>
              <div className='flex items-center gap-3'>
                <div className='bg-destructive/10 text-destructive p-2.5 rounded-lg'>
                  <Trash2 className='h-5 w-5' />
                </div>
                <div>
                  <p className='font-semibold text-base'>
                    {selectedItems.length} {selectedItems.length === 1 ? 'item' : 'items'} selected
                  </p>
                  <p className='text-sm text-muted-foreground'>
                    {isConfirmingDelete ? 'Click confirm to delete permanently' : 'Ready to delete?'}
                  </p>
                </div>
              </div>
              <div className='flex gap-2'>
                <Button
                  variant='ghost'
                  size='sm'
                  onClick={() => {
                    setSelectedItems([])
                    setIsConfirmingDelete(false)
                  }}
                  className='hover:bg-muted'
                >
                  Cancel
                </Button>
                <Button
                  variant='destructive'
                  size='sm'
                  onClick={
                    isConfirmingDelete ? handleConfirmDelete : () => setIsConfirmingDelete(true)
                  }
                  className='min-w-[80px]'
                >
                  {isConfirmingDelete ? 'Confirm' : 'Delete'}
                </Button>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}

function TableSkeleton({
  columns,
  enableSelection,
}: {
  columns: number
  enableSelection?: boolean
}) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          {enableSelection && (
            <TableHead className='w-[40px]'>
              <Skeleton className='h-4 w-4' />
            </TableHead>
          )}
          {Array.from({ length: columns }).map((_, i) => (
            <TableHead key={i}>
              <Skeleton className='h-4 w-[80px]' />
            </TableHead>
          ))}
          <TableHead className='w-[80px]'>
            <Skeleton className='h-4 w-[40px]' />
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {Array.from({ length: 5 }).map((_, rowIndex) => (
          <TableRow key={rowIndex}>
            {enableSelection && (
              <TableCell>
                <Skeleton className='h-4 w-4' />
              </TableCell>
            )}
            {Array.from({ length: columns }).map((_, colIndex) => (
              <TableCell key={colIndex}>
                <Skeleton className='h-4 w-full' />
              </TableCell>
            ))}
            <TableCell>
              <Skeleton className='h-8 w-8' />
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}
