import { useTranslation } from 'react-i18next'
import { enUS, zhCN } from 'react-day-picker/locale'
import { Calendar } from '@/components/ui/calendar'
import { getActiveLocale } from '@/lib/i18n'
import type { SupportedLocale } from '@/lib/i18n/locales'
import { cn } from '@/lib/utils'
import { boundsFromCalendarRange, calendarRangeFromBounds } from './date-range-bounds'
import type { PagedListing } from './use-paged-listing'

const CALENDAR_LOCALES = { en: enUS, 'zh-CN': zhCN } satisfies Record<SupportedLocale, unknown>

const MONDAY = 1

const RANGE_SEPARATOR = ' – '

const CALENDAR_CLASS = 'mx-auto p-0 [--cell-size:--spacing(7)]'

const CALENDAR_CLASS_NAMES = {
  caption_label: 'text-xs font-medium select-none',
  weekday: 'flex-1 text-[11px] font-normal text-muted-foreground select-none',
  week: 'mt-1 flex w-full',
  day_button: 'text-xs',
}

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
  const locale = getActiveLocale()
  const stored = {
    from: listing.state.filters[fromKey] ?? '',
    to: listing.state.filters[toKey] ?? '',
  }
  const selected = calendarRangeFromBounds(stored.from, stored.to)
  const format = new Intl.DateTimeFormat(locale, { dateStyle: 'medium' })

  return (
    <div className='flex flex-col gap-1'>
      <Calendar
        mode='range'
        aria-label={label}
        numberOfMonths={1}
        weekStartsOn={MONDAY}
        locale={CALENDAR_LOCALES[locale]}
        defaultMonth={selected?.from ?? selected?.to}
        selected={selected}
        onSelect={(range) => {
          const bounds = boundsFromCalendarRange(stored, range)
          listing.setFilters({ [fromKey]: bounds.from, [toKey]: bounds.to })
        }}
        className={CALENDAR_CLASS}
        classNames={CALENDAR_CLASS_NAMES}
      />
      <p className='tnum text-center text-[11px] text-neutral-500 dark:text-neutral-400'>
        <span className={cn(selected?.from && 'text-neutral-900 dark:text-neutral-100')}>
          {selected?.from ? format.format(selected.from) : t('column_filter.from')}
        </span>
        {RANGE_SEPARATOR}
        <span className={cn(selected?.to && 'text-neutral-900 dark:text-neutral-100')}>
          {selected?.to ? format.format(selected.to) : t('column_filter.to')}
        </span>
      </p>
    </div>
  )
}
