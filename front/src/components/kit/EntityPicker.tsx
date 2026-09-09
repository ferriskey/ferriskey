import { useState } from 'react'
import { Check, ChevronsUpDown, Plus, Trash2 } from 'lucide-react'
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
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

export interface PickableEntity {
  id: string
  label: string
  sublabel?: string
}

export function EntityPicker({
  items,
  value,
  onChange,
  addLabel = 'Add',
  searchPlaceholder = 'Search…',
  emptyHint = 'None configured.',
  exhaustedHint = 'Everything is already added.',
  disabled,
}: {
  items: PickableEntity[]
  value: string[]
  onChange: (next: string[]) => void
  addLabel?: string
  searchPlaceholder?: string
  emptyHint?: string
  exhaustedHint?: string
  disabled?: boolean
}) {
  const [open, setOpen] = useState(false)

  const selected = value
    .map((id) => items.find((i) => i.id === id))
    .filter((i): i is PickableEntity => Boolean(i))
  const available = items.filter((i) => !value.includes(i.id))

  return (
    <div className='max-w-lg space-y-2'>
      {selected.length > 0 ? (
        <ul className={cn(tokens.surface.panel, 'divide-y divide-fk-line-soft')}>
          {selected.map((entity) => (
            <li key={entity.id} className='flex items-center gap-3 px-3 py-2'>
              <div className='min-w-0 flex-1'>
                <p className='truncate text-xs font-medium text-neutral-900'>
                  {entity.label}
                </p>
                {entity.sublabel && (
                  <p className='truncate font-mono-ui text-[11px] text-neutral-500'>
                    {entity.sublabel}
                  </p>
                )}
              </div>
              <Button
                variant='ghost'
                size='icon'
                disabled={disabled}
                aria-label={`Remove ${entity.label}`}
                onClick={() => onChange(value.filter((id) => id !== entity.id))}
                className='size-7 text-neutral-400 hover:text-fk-danger'
              >
                <Trash2 />
              </Button>
            </li>
          ))}
        </ul>
      ) : (
        <p className='text-xs text-neutral-500'>{emptyHint}</p>
      )}

      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <Button
            type='button'
            variant='outline'
            size='sm'
            disabled={disabled || available.length === 0}
            aria-expanded={open}
          >
            <Plus /> {addLabel}
            <ChevronsUpDown className='text-neutral-400' />
          </Button>
        </PopoverTrigger>
        <PopoverContent align='start' className='w-72 p-0'>
          <Command>
            <CommandInput placeholder={searchPlaceholder} />
            <CommandList>
              <CommandEmpty>No match.</CommandEmpty>
              <CommandGroup>
                {available.map((entity) => (
                  <CommandItem
                    key={entity.id}
                    value={`${entity.label} ${entity.sublabel ?? ''}`}
                    onSelect={() => {
                      onChange([...value, entity.id])
                      setOpen(false)
                    }}
                  >
                    <Check className='opacity-0' />
                    <span className='min-w-0 flex-1'>
                      <span className='block truncate text-xs text-neutral-900'>
                        {entity.label}
                      </span>
                      {entity.sublabel && (
                        <span className='block truncate font-mono-ui text-[11px] text-neutral-400'>
                          {entity.sublabel}
                        </span>
                      )}
                    </span>
                  </CommandItem>
                ))}
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>

      {available.length === 0 && selected.length > 0 && (
        <p className='text-xs text-neutral-400'>{exhaustedHint}</p>
      )}
    </div>
  )
}
