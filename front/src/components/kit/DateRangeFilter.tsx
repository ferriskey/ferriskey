import { useTranslation } from 'react-i18next'
import { fromRangeBounds, toRangeBounds } from './date-range-bounds'
import type { PagedListing } from './use-paged-listing'

const DATE_INPUT_CLASS =
  'h-8 w-full rounded-md border border-fk-line bg-white px-2 text-sm outline-none focus:border-fk-primary-border focus:ring-2 focus:ring-fk-primary/15 dark:bg-fk-surface'

export function DateRangeFilter({
  fromKey,
  toKey,
  label,
  listing,
}: {
  fromKey: string
  toKey: string
  label: string
  listing: PagedListing
}) {
  const { t } = useTranslation()
  const { fromDay, toDay } = fromRangeBounds(
    listing.state.filters[fromKey] ?? '',
    listing.state.filters[toKey] ?? '',
  )

  const write = (nextFrom: string, nextTo: string) => {
    const bounds = toRangeBounds(nextFrom, nextTo)
    listing.setFilters({ [fromKey]: bounds.from, [toKey]: bounds.to })
  }

  return (
    <div className='grid grid-cols-2 gap-2'>
      <label className='flex flex-col gap-1'>
        <span className='text-[11px] text-neutral-500 dark:text-neutral-400'>
          {t('column_filter.from')}
        </span>
        <input
          type='date'
          aria-label={`${label} ${t('column_filter.from')}`}
          value={fromDay}
          max={toDay || undefined}
          onChange={(e) => write(e.target.value, toDay)}
          className={DATE_INPUT_CLASS}
        />
      </label>
      <label className='flex flex-col gap-1'>
        <span className='text-[11px] text-neutral-500 dark:text-neutral-400'>
          {t('column_filter.to')}
        </span>
        <input
          type='date'
          aria-label={`${label} ${t('column_filter.to')}`}
          value={toDay}
          min={fromDay || undefined}
          onChange={(e) => write(fromDay, e.target.value)}
          className={DATE_INPUT_CLASS}
        />
      </label>
    </div>
  )
}
