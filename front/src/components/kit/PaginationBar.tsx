import { Trans, useTranslation } from 'react-i18next'
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-react'
import { Button } from '@/components/kit/button'
import type { PaginationMetadata } from './listing-query-state'

const NUM = { num: <span className='tnum font-medium text-neutral-700 dark:text-neutral-300' /> }

export function PaginationBar({
  pagination,
  onPageChange,
}: {
  pagination: PaginationMetadata
  onPageChange: (page: number) => void
}) {
  const { t } = useTranslation()
  const { page, limit, total, last_page, first_page } = pagination
  const prev = pagination.prev_page ?? null
  const next = pagination.next_page ?? null
  const from = total === 0 ? 0 : (page - 1) * limit + 1
  const to = Math.min(page * limit, total)

  return (
    <div className='flex flex-wrap items-center justify-between gap-3 text-xs text-neutral-500 dark:text-neutral-400'>
      <p>
        <Trans
          i18nKey='data_view.pagination.range'
          values={{ from, to, total }}
          components={NUM}
        />
      </p>
      <div className='flex items-center gap-2'>
        <p>
          <Trans
            i18nKey='data_view.pagination.page'
            values={{ current: page, total: last_page }}
            components={NUM}
          />
        </p>
        <div className='flex items-center gap-1'>
          <Button
            variant='outline'
            size='icon-sm'
            aria-label={t('data_view.pagination.first')}
            disabled={prev === null}
            onClick={() => onPageChange(first_page)}
          >
            <ChevronsLeft />
          </Button>
          <Button
            variant='outline'
            size='icon-sm'
            aria-label={t('data_view.pagination.previous')}
            disabled={prev === null}
            onClick={() => prev !== null && onPageChange(prev)}
          >
            <ChevronLeft />
          </Button>
          <Button
            variant='outline'
            size='icon-sm'
            aria-label={t('data_view.pagination.next')}
            disabled={next === null}
            onClick={() => next !== null && onPageChange(next)}
          >
            <ChevronRight />
          </Button>
          <Button
            variant='outline'
            size='icon-sm'
            aria-label={t('data_view.pagination.last')}
            disabled={next === null}
            onClick={() => onPageChange(last_page)}
          >
            <ChevronsRight />
          </Button>
        </div>
      </div>
    </div>
  )
}
