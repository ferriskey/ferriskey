import { useEffect, useState } from 'react'
import { Check, ChevronsUpDown, Loader2, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Pill } from '@/components/kit'
import { cn } from '@/lib/utils'

export interface MapperEntityOption {
  value: string
  label: string
  sublabel?: string
}

export interface MapperEntitySource {
  useOptions: (search: string) => { options: MapperEntityOption[]; loading: boolean }
  useSelected: (value: string) => { option: MapperEntityOption | undefined; loading: boolean }
}

const SEARCH_DEBOUNCE_MS = 300

export interface MapperEntityFieldProps {
  id: string
  source: MapperEntitySource
  value: string
  placeholder: string
  searchPlaceholder: string
  emptyLabel: string
  onChange: (value: string) => void
}

export default function MapperEntityField({
  id,
  source,
  value,
  placeholder,
  searchPlaceholder,
  emptyLabel,
  onChange,
}: MapperEntityFieldProps) {
  const { t } = useTranslation('client-scope')
  const [open, setOpen] = useState(false)
  const [search, setSearch] = useState('')
  const [debounced, setDebounced] = useState('')
  const { options, loading } = source.useOptions(debounced)
  const { option: selected, loading: resolving } = source.useSelected(value)

  useEffect(() => {
    const timer = setTimeout(() => setDebounced(search), SEARCH_DEBOUNCE_MS)
    return () => clearTimeout(timer)
  }, [search])

  const changeOpen = (next: boolean) => {
    setOpen(next)
    if (next) return
    setSearch('')
    setDebounced('')
  }

  return (
    <div className='flex max-w-sm items-center gap-1.5'>
      <Popover open={open} onOpenChange={changeOpen}>
        <PopoverTrigger asChild>
          <Button
            id={id}
            type='button'
            variant='outline'
            role='combobox'
            aria-expanded={open}
            className='w-full justify-between font-normal'
          >
            {value ? (
              <span className='flex min-w-0 items-center gap-2'>
                <span className='min-w-0 truncate font-mono-ui text-xs'>{value}</span>
                {!selected && !resolving && (
                  <Pill tone='amber'>{t('mapper_entity.unknown')}</Pill>
                )}
              </span>
            ) : (
              <span className='text-neutral-500 dark:text-neutral-400'>{placeholder}</span>
            )}
            <ChevronsUpDown className='opacity-50' />
          </Button>
        </PopoverTrigger>
        <PopoverContent className='w-(--radix-popover-trigger-width) p-0' align='start'>
          <Command shouldFilter={false}>
            <CommandInput
              placeholder={searchPlaceholder}
              value={search}
              onValueChange={setSearch}
              className='h-9'
            />
            <CommandList>
              {loading ? (
                <div className='flex justify-center py-6'>
                  <Loader2 className='size-4 animate-spin text-neutral-400' />
                </div>
              ) : (
                <CommandEmpty>{emptyLabel}</CommandEmpty>
              )}
              <CommandGroup>
                {options.map((option) => (
                  <CommandItem
                    key={option.value}
                    value={`${option.label} ${option.sublabel ?? ''} ${option.value}`}
                    onSelect={() => {
                      onChange(option.value)
                      changeOpen(false)
                    }}
                  >
                    <span className='min-w-0 flex-1'>
                      <span className='block truncate text-xs text-neutral-900 dark:text-neutral-100'>
                        {option.label}
                      </span>
                      {option.sublabel && (
                        <span className='block truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
                          {option.sublabel}
                        </span>
                      )}
                    </span>
                    <Check
                      className={cn('ml-auto', option.value === value ? 'opacity-100' : 'opacity-0')}
                    />
                  </CommandItem>
                ))}
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>

      {value && (
        <Button
          type='button'
          variant='ghost'
          size='icon'
          aria-label={t('mapper_entity.clear')}
          className='size-8 shrink-0 text-neutral-400 dark:text-neutral-500'
          onClick={() => onChange('')}
        >
          <X />
        </Button>
      )}
    </div>
  )
}
