import { useState } from 'react'
import type { ComponentType } from 'react'
import { useTranslation } from 'react-i18next'
import { ChevronsUpDown, Plus, Trash2, Users } from 'lucide-react'
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
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { EmptyState } from './empty-state'

export interface PickableEntity {
  id: string
  label: string
  sublabel?: string
}

export function EntityPicker({
  items,
  value,
  onChange,
  addLabel,
  searchPlaceholder,
  emptyHint,
  emptyIcon = Users,
  exhaustedHint,
  disabled,
  fullWidth = false,
}: {
  items: PickableEntity[]
  value: string[]
  onChange: (next: string[]) => void
  addLabel?: string
  searchPlaceholder?: string
  emptyHint?: string
  emptyIcon?: ComponentType<{ className?: string; strokeWidth?: number }>
  exhaustedHint?: string
  disabled?: boolean
  fullWidth?: boolean
}) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const addText = addLabel ?? t('action.add')
  const searchText = searchPlaceholder ?? t('entity_picker.search_placeholder')
  const emptyText = emptyHint ?? t('entity_picker.empty')
  const exhaustedText = exhaustedHint ?? t('entity_picker.exhausted')

  const selected = value
    .map((id) => items.find((i) => i.id === id))
    .filter((i): i is PickableEntity => Boolean(i))
  const available = items.filter((i) => !value.includes(i.id))

  const addControl = (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          type='button'
          variant='outline'
          size='sm'
          disabled={disabled || available.length === 0}
          aria-expanded={open}
        >
          <Plus /> {addText}
          <ChevronsUpDown className='text-neutral-400 dark:text-neutral-500' />
        </Button>
      </PopoverTrigger>
      <PopoverContent align='start' sideOffset={6} className='w-64 p-0'>
        <Command>
          <CommandInput
            placeholder={searchText}
            className='h-8 py-0 text-[13px]'
          />
          <CommandList className='max-h-56'>
            <CommandEmpty className='py-3 text-[13px]'>{t('entity_picker.no_match')}</CommandEmpty>
            <CommandGroup className='p-1'>
              {available.map((entity) => (
                <CommandItem
                  key={entity.id}
                  value={`${entity.label} ${entity.sublabel ?? ''}`}
                  className='px-1.5 py-1'
                  onSelect={() => {
                    onChange([...value, entity.id])
                    setOpen(false)
                  }}
                >
                  <span className='min-w-0 flex-1'>
                    <span className='block truncate text-[13px] text-neutral-900 dark:text-neutral-100'>
                      {entity.label}
                    </span>
                    {entity.sublabel && (
                      <span className='block truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
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
  )

  if (selected.length === 0) {
    const exhausted = available.length === 0
    return (
      <div className={fullWidth ? undefined : 'max-w-lg'}>
        <EmptyState
          icon={emptyIcon}
          compact
          label={exhausted ? exhaustedText : emptyText}
          action={exhausted ? undefined : addControl}
        />
      </div>
    )
  }

  return (
    <div className={cn('space-y-2', !fullWidth && 'max-w-lg')}>
      {selected.length > 0 ? (
        <ul className={cn(tokens.surface.panel, 'divide-y divide-fk-line-soft')}>
          {selected.map((entity) => (
            <li key={entity.id} className='flex items-center gap-3 px-3 py-2'>
              <div className='min-w-0 flex-1'>
                <p className='truncate text-xs font-medium text-neutral-900 dark:text-neutral-100'>
                  {entity.label}
                </p>
                {entity.sublabel && (
                  <p className='truncate font-mono-ui text-[11px] text-neutral-500 dark:text-neutral-400'>
                    {entity.sublabel}
                  </p>
                )}
              </div>
              <Button
                variant='ghost'
                size='icon'
                disabled={disabled}
                aria-label={t('entity_picker.remove', { label: entity.label })}
                onClick={() => onChange(value.filter((id) => id !== entity.id))}
                className='size-7 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
              >
                <Trash2 />
              </Button>
            </li>
          ))}
        </ul>
      ) : null}

      {addControl}

      {available.length === 0 && (
        <p className='text-xs text-neutral-400 dark:text-neutral-500'>{exhaustedText}</p>
      )}
    </div>
  )
}
