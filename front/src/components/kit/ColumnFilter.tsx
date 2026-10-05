import { useTranslation } from 'react-i18next'
import { Filter, X } from 'lucide-react'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn } from '@/lib/utils'
import { Button } from './button'
import {
  activeFieldCount,
  clearColumnFilters,
  columnFilterIndicator,
  fieldKeys,
  toggleOptionCard,
  usesOptionCards,
  type ColumnFilterField,
  type ColumnFilterIndicator,
} from './column-filter-state'
import { DateRangeFilter } from './DateRangeFilter'
import { RelationSelect } from './RelationSelect'
import type { PagedListing } from './use-paged-listing'

const ANY_VALUE = '__any__'
const TRUE_VALUE = 'true'
const FALSE_VALUE = 'false'

const CONTROL_CLASS =
  'h-7 w-full rounded-md border border-neutral-300 bg-white text-[13px] text-neutral-900 dark:border-neutral-600 dark:bg-fk-canvas dark:text-neutral-100 outline-none focus:border-fk-primary-border focus:ring-2 focus:ring-fk-primary/15'

function OptionCards({
  label,
  value,
  options,
  onChange,
}: {
  label: string
  value: string | undefined
  options: { value: string; label: string }[]
  onChange: (value: string) => void
}) {
  return (
    <div
      role='group'
      aria-label={label}
      className='flex h-7 w-full divide-x divide-neutral-300 overflow-hidden rounded-md border border-neutral-300 bg-white dark:divide-neutral-600 dark:border-neutral-600 dark:bg-fk-canvas'
    >
      {options.map((option) => {
        const selected = value === option.value
        return (
          <button
            key={option.value}
            type='button'
            aria-pressed={selected}
            onClick={() => onChange(toggleOptionCard(value, option.value))}
            className={cn(
              'min-w-0 flex-1 cursor-pointer truncate px-1.5 text-[12px] transition-colors outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-fk-primary/30',
              selected
                ? 'bg-fk-primary-soft font-medium text-fk-primary-text'
                : 'text-neutral-600 hover:bg-neutral-50 dark:text-neutral-300 dark:hover:bg-white/5'
            )}
          >
            {option.label}
          </button>
        )
      })}
    </div>
  )
}

function ChoiceControl({
  label,
  value,
  options,
  onChange,
}: {
  label: string
  value: string | undefined
  options: { value: string; label: string }[]
  onChange: (value: string) => void
}) {
  const { t } = useTranslation()
  return (
    <Select
      value={value || ANY_VALUE}
      onValueChange={(next) => onChange(next === ANY_VALUE ? '' : next)}
    >
      <SelectTrigger
        size='sm'
        aria-label={label}
        className={cn(CONTROL_CLASS, 'px-2 shadow-none', value && 'border-fk-primary-border')}
      >
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={ANY_VALUE}>{t('filter_bar.any')}</SelectItem>
        {options.map((option) => (
          <SelectItem key={option.value} value={option.value}>
            {option.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}

function FieldControl({ field, listing }: { field: ColumnFilterField; listing: PagedListing }) {
  const { t } = useTranslation()

  switch (field.kind) {
    case 'boolean':
    case 'enum': {
      const options =
        field.kind === 'boolean'
          ? [
              { value: TRUE_VALUE, label: t('column_filter.yes') },
              { value: FALSE_VALUE, label: t('column_filter.no') },
            ]
          : field.options
      const Control = usesOptionCards(field) ? OptionCards : ChoiceControl
      return (
        <Control
          label={field.label}
          value={listing.state.filters[field.key]}
          onChange={(value) => listing.setFilter(field.key, value)}
          options={options}
        />
      )
    }
    case 'relation':
      return (
        <RelationSelect
          source={field.relation}
          value={listing.state.filters[field.key] || undefined}
          onChange={(id) => listing.setFilter(field.key, id ?? '')}
          label={field.label}
          compact
        />
      )
    case 'date-range':
      return (
        <DateRangeFilter
          fromKey={field.fromKey}
          toKey={field.toKey}
          label={field.label}
          listing={listing}
        />
      )
  }
}

function IndicatorMark({ indicator }: { indicator: ColumnFilterIndicator }) {
  const { t } = useTranslation()
  switch (indicator.kind) {
    case 'count':
      return (
        <span className='tnum inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-fk-primary px-1 text-[10px] font-semibold leading-none text-white'>
          {indicator.count}
        </span>
      )
    case 'value':
      return (
        <span className='inline-flex h-4 items-center rounded-full bg-fk-primary-soft px-1.5 text-[10px] font-medium leading-none text-fk-primary-text'>
          {t(indicator.value === TRUE_VALUE ? 'column_filter.yes' : 'column_filter.no')}
        </span>
      )
    case 'active':
      return <Filter className='size-3 fill-current text-fk-primary-text' />
    case 'none':
      return <Filter className='size-3 text-neutral-400 dark:text-neutral-500' />
  }
}

export function ColumnFilter({
  fields,
  listing,
  label,
  showLabel = false,
}: {
  fields: ColumnFilterField[]
  listing: PagedListing
  label: string
  showLabel?: boolean
}) {
  const { t } = useTranslation()
  const indicator = columnFilterIndicator(fields, listing.state.filters)
  const engaged = indicator.kind !== 'none'
  const opener = `${t('column_filter.open')} ${label}`
  const announced =
    indicator.kind === 'count'
      ? t('column_filter.state_count', { label: opener, count: indicator.count })
      : indicator.kind === 'value'
        ? t('column_filter.state_value', {
            label: opener,
            value: t(indicator.value === TRUE_VALUE ? 'column_filter.yes' : 'column_filter.no'),
          })
        : indicator.kind === 'active'
          ? t('column_filter.state_active', { label: opener })
          : opener

  return (
    <Popover>
      <PopoverTrigger asChild>
        <button
          type='button'
          aria-label={announced}
          className={cn(
            'inline-flex cursor-pointer items-center gap-1 rounded transition-colors hover:text-neutral-700 dark:hover:text-neutral-300',
            showLabel
              ? 'h-7 rounded-md border border-fk-line px-2 text-xs'
              : 'h-5 min-w-5 justify-center px-0.5',
            showLabel && engaged && 'border-fk-primary-border text-fk-primary-text'
          )}
        >
          {showLabel && <span>{label}</span>}
          <IndicatorMark indicator={indicator} />
        </button>
      </PopoverTrigger>
      <PopoverContent align='start' sideOffset={4} className='w-52 space-y-1.5 border-neutral-300 bg-white p-1.5 shadow-lg shadow-black/10 dark:border-neutral-700 dark:bg-fk-raised dark:shadow-black/50'>
        {fields.map((field) => {
          const active = activeFieldCount([field], listing.state.filters) > 0
          return (
            <div key={fieldKeys(field).join(':')} className='flex flex-col gap-0.5'>
              <div className='flex h-5 items-center justify-between gap-2 px-0.5'>
                <span className='text-[11px] font-medium text-neutral-600 dark:text-neutral-300'>
                  {field.label}
                </span>
                {active && (
                  <Button
                    variant='ghost'
                    size='icon-xs'
                    className='size-5'
                    aria-label={`${t('column_filter.clear')} ${field.label}`}
                    onClick={() => listing.setFilters(clearColumnFilters([field]))}
                  >
                    <X />
                  </Button>
                )}
              </div>
              <FieldControl field={field} listing={listing} />
            </div>
          )
        })}
      </PopoverContent>
    </Popover>
  )
}
