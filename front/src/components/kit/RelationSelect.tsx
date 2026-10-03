import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ChevronsUpDown, Loader2 } from 'lucide-react'
import {
  Command,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { cn } from '@/lib/utils'

export interface RelationOption {
  id: string
  label: string
  sublabel?: string
}

export interface RelationSource {
  useOptions: (search: string) => { options: RelationOption[]; loading: boolean }
  useSelected: (id: string | undefined) => RelationOption | undefined
}

const SEARCH_DEBOUNCE_MS = 300
const ANY_ITEM_VALUE = '__any__'

function useDebounced(value: string, delay: number): string {
  const [debounced, setDebounced] = useState(value)
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delay)
    return () => clearTimeout(timer)
  }, [value, delay])
  return debounced
}

export function RelationSelect({
  source,
  value,
  onChange,
  label,
}: {
  source: RelationSource
  value: string | undefined
  onChange: (id: string | undefined) => void
  label: string
}) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const [search, setSearch] = useState('')
  const debounced = useDebounced(search, SEARCH_DEBOUNCE_MS)
  const { options, loading } = source.useOptions(debounced)
  const selected = source.useSelected(value)
  const shown = value ? (selected?.label ?? value) : t('filter_bar.any')

  const pick = (id: string | undefined) => {
    onChange(id)
    setOpen(false)
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          type='button'
          aria-expanded={open}
          className={cn(
            'inline-flex h-8 max-w-[16rem] cursor-pointer items-center gap-1.5 rounded-md border border-fk-line bg-white px-2.5 text-sm outline-none focus:border-fk-primary-border focus:ring-2 focus:ring-fk-primary/15 dark:bg-fk-surface',
            value && 'border-fk-primary-border'
          )}
        >
          <span className='shrink-0 text-neutral-500 dark:text-neutral-400'>{label}:</span>
          <span className='min-w-0 truncate text-neutral-900 dark:text-neutral-100'>{shown}</span>
          <ChevronsUpDown className='size-3.5 shrink-0 text-neutral-400 dark:text-neutral-500' />
        </button>
      </PopoverTrigger>
      <PopoverContent align='start' sideOffset={6} className='w-64 p-0'>
        <Command shouldFilter={false}>
          <CommandInput
            value={search}
            onValueChange={setSearch}
            placeholder={t('filter_bar.search')}
            className='h-8 py-0 text-[13px]'
          />
          <CommandList className='max-h-56'>
            <CommandGroup className='p-1'>
              <CommandItem
                value={ANY_ITEM_VALUE}
                className='px-1.5 py-1 text-[13px]'
                onSelect={() => pick(undefined)}
              >
                {t('filter_bar.any')}
              </CommandItem>
              {options.map((option) => (
                <CommandItem
                  key={option.id}
                  value={option.id}
                  className='px-1.5 py-1'
                  onSelect={() => pick(option.id)}
                >
                  <span className='min-w-0 flex-1'>
                    <span className='block truncate text-[13px] text-neutral-900 dark:text-neutral-100'>
                      {option.label}
                    </span>
                    {option.sublabel && (
                      <span className='block truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
                        {option.sublabel}
                      </span>
                    )}
                  </span>
                </CommandItem>
              ))}
            </CommandGroup>
            {loading && (
              <div className='flex justify-center py-3'>
                <Loader2 className='size-4 animate-spin text-neutral-400 dark:text-neutral-500' />
              </div>
            )}
            {!loading && options.length === 0 && (
              <p className='py-3 text-center text-[13px] text-neutral-500 dark:text-neutral-400'>
                {t('entity_picker.empty')}
              </p>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}
