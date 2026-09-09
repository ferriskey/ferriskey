import { useState } from 'react'
import { Check, ChevronsUpDown, X } from 'lucide-react'
import { Button } from '@/components/ui/button'
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

export interface MapperEntityFieldProps {
  id: string
  options: MapperEntityOption[]
  value: string
  placeholder: string
  searchPlaceholder: string
  emptyLabel: string
  onChange: (value: string) => void
}

export default function MapperEntityField({
  id,
  options,
  value,
  placeholder,
  searchPlaceholder,
  emptyLabel,
  onChange,
}: MapperEntityFieldProps) {
  const [open, setOpen] = useState(false)
  const selected = options.find((option) => option.value === value)

  return (
    <div className='flex max-w-sm items-center gap-1.5'>
      <Popover open={open} onOpenChange={setOpen}>
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
                {!selected && <Pill tone='amber'>unknown</Pill>}
              </span>
            ) : (
              <span className='text-neutral-500'>{placeholder}</span>
            )}
            <ChevronsUpDown className='opacity-50' />
          </Button>
        </PopoverTrigger>
        <PopoverContent className='w-(--radix-popover-trigger-width) p-0' align='start'>
          <Command>
            <CommandInput placeholder={searchPlaceholder} className='h-9' />
            <CommandList>
              <CommandEmpty>{emptyLabel}</CommandEmpty>
              <CommandGroup>
                {options.map((option) => (
                  <CommandItem
                    key={option.value}
                    value={`${option.label} ${option.sublabel ?? ''} ${option.value}`}
                    onSelect={() => {
                      onChange(option.value)
                      setOpen(false)
                    }}
                  >
                    <span className='min-w-0 flex-1'>
                      <span className='block truncate text-xs text-neutral-900'>
                        {option.label}
                      </span>
                      {option.sublabel && (
                        <span className='block truncate font-mono-ui text-[11px] text-neutral-400'>
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
          aria-label='Clear'
          className='size-8 shrink-0 text-neutral-400'
          onClick={() => onChange('')}
        >
          <X />
        </Button>
      )}
    </div>
  )
}
