import { useTranslation } from 'react-i18next'
import { Search, X } from 'lucide-react'
import { Button } from '@/components/kit/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn } from '@/lib/utils'
import { RelationSelect, type RelationSource } from './RelationSelect'
import type { PagedListing } from './use-paged-listing'

export type FilterField =
  | { kind: 'text'; key: string; label: string; placeholder?: string }
  | { kind: 'boolean'; key: string; label: string }
  | { kind: 'enum'; key: string; label: string; options: { value: string; label: string }[] }
  | { kind: 'relation'; key: string; label: string; relation: RelationSource }

const ANY_VALUE = '__any__'
const TRUE_VALUE = 'true'
const FALSE_VALUE = 'false'

const CONTROL_CLASS =
  'h-8 rounded-md border border-fk-line bg-white dark:bg-fk-surface text-sm outline-none focus:border-fk-primary-border focus:ring-2 focus:ring-fk-primary/15'

function ChoiceFilter({
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
        className={cn(CONTROL_CLASS, 'gap-1.5 px-2.5 shadow-none', value && 'border-fk-primary-border')}
      >
        <span className='text-neutral-500 dark:text-neutral-400'>{label}:</span>
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

function FilterControl({ field, listing }: { field: FilterField; listing: PagedListing }) {
  const { t } = useTranslation()
  const current = listing.state.filters[field.key]
  const setFilter = (value: string) => listing.setFilter(field.key, value)

  switch (field.kind) {
    case 'text':
      return (
        <label className='relative flex h-8 min-w-[15rem] flex-1 items-center'>
          <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400 dark:text-neutral-500' />
          <input
            type='search'
            aria-label={field.label}
            value={listing.drafts[field.key] ?? ''}
            onChange={(e) => listing.setDraft(field.key, e.target.value)}
            placeholder={field.placeholder ?? field.label}
            className={cn(CONTROL_CLASS, 'h-full w-full pl-8 pr-3 placeholder:text-neutral-400')}
          />
        </label>
      )
    case 'boolean':
      return (
        <ChoiceFilter
          label={field.label}
          value={current}
          onChange={setFilter}
          options={[
            { value: TRUE_VALUE, label: t('filter_bar.yes') },
            { value: FALSE_VALUE, label: t('filter_bar.no') },
          ]}
        />
      )
    case 'enum':
      return (
        <ChoiceFilter
          label={field.label}
          value={current}
          onChange={setFilter}
          options={field.options}
        />
      )
    case 'relation':
      return (
        <RelationSelect
          source={field.relation}
          value={current}
          onChange={(id) => setFilter(id ?? '')}
          label={field.label}
        />
      )
  }
}

export function FilterBar({ fields, listing }: { fields: FilterField[]; listing: PagedListing }) {
  const { t } = useTranslation()
  const anySet = Object.values(listing.state.filters).some(Boolean)

  return (
    <div className='flex flex-1 flex-wrap items-center gap-2'>
      {fields.map((field) => (
        <FilterControl key={field.key} field={field} listing={listing} />
      ))}
      {anySet && (
        <Button variant='ghost' size='sm' onClick={listing.clearFilters}>
          <X />
          {t('filter_bar.clear')}
        </Button>
      )}
    </div>
  )
}
