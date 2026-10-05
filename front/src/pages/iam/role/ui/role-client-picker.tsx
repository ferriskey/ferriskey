import { useState } from 'react'
import { Check, ChevronsUpDown, Loader2 } from 'lucide-react'
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
import type { ClientPicking } from '@/hooks/use-client-picker'

export interface RoleClientPickerProps {
  picker: ClientPicking
  value?: string
  onChange: (clientId: string) => void
}

export default function RoleClientPicker({ picker, value, onChange }: RoleClientPickerProps) {
  const { t } = useTranslation('role')
  const [open, setOpen] = useState(false)
  const selected = value
    ? (picker.clients.find((c) => c.id === value) ??
      (picker.selected?.id === value ? picker.selected : undefined))
    : undefined

  const changeOpen = (next: boolean) => {
    setOpen(next)
    if (!next) picker.onSearchChange('')
  }

  return (
    <Popover open={open} onOpenChange={changeOpen}>
      <PopoverTrigger asChild>
        <Button
          type='button'
          variant='outline'
          role='combobox'
          aria-expanded={open}
          className='w-full max-w-sm justify-between font-normal'
        >
          {selected ? (
            <span className='min-w-0 truncate'>
              {selected.name}
              <span className='ml-2 font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
                {selected.client_id}
              </span>
            </span>
          ) : (
            <span className='text-neutral-500 dark:text-neutral-400'>
              {t('form.client.placeholder')}
            </span>
          )}
          <ChevronsUpDown className='opacity-50' />
        </Button>
      </PopoverTrigger>
      <PopoverContent className='w-(--radix-popover-trigger-width) p-0' align='start'>
        <Command shouldFilter={false}>
          <CommandInput
            value={picker.search}
            onValueChange={picker.onSearchChange}
            placeholder={t('form.client.search_placeholder')}
            className='h-9'
          />
          <CommandList>
            {!picker.loading && <CommandEmpty>{t('form.client.empty')}</CommandEmpty>}
            <CommandGroup>
              {picker.clients.map((client) => (
                <CommandItem
                  key={client.id}
                  value={client.id}
                  onSelect={() => {
                    onChange(client.id)
                    changeOpen(false)
                  }}
                >
                  <span className='min-w-0 flex-1'>
                    <span className='block truncate text-xs text-neutral-900 dark:text-neutral-100'>
                      {client.name}
                    </span>
                    <span className='block truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
                      {client.client_id}
                    </span>
                  </span>
                  {value === client.id && <Check className='ml-auto' />}
                </CommandItem>
              ))}
            </CommandGroup>
            {picker.loading && (
              <div className='flex justify-center py-3'>
                <Loader2 className='size-4 animate-spin text-neutral-400 dark:text-neutral-500' />
              </div>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}
