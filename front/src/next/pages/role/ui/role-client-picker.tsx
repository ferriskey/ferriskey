import { useState } from 'react'
import { Check, ChevronsUpDown } from 'lucide-react'
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
import { Schemas } from '@/api/api.client'

import Client = Schemas.Client

export interface RoleClientPickerProps {
  clients: Client[]
  value?: string
  onChange: (clientId: string) => void
}

export default function RoleClientPicker({ clients, value, onChange }: RoleClientPickerProps) {
  const [open, setOpen] = useState(false)
  const selected = clients.find((c) => c.id === value)

  return (
    <Popover open={open} onOpenChange={setOpen}>
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
              <span className='ml-2 font-mono-ui text-xs text-neutral-400'>
                {selected.client_id}
              </span>
            </span>
          ) : (
            <span className='text-neutral-500'>Select a client…</span>
          )}
          <ChevronsUpDown className='opacity-50' />
        </Button>
      </PopoverTrigger>
      <PopoverContent className='w-(--radix-popover-trigger-width) p-0' align='start'>
        <Command>
          <CommandInput placeholder='Search a client…' className='h-9' />
          <CommandList>
            <CommandEmpty>No client found.</CommandEmpty>
            <CommandGroup>
              {clients.map((client) => (
                <CommandItem
                  key={client.id}
                  value={`${client.name} ${client.client_id}`}
                  onSelect={() => {
                    onChange(client.id)
                    setOpen(false)
                  }}
                >
                  <span className='min-w-0 flex-1'>
                    <span className='block truncate text-xs text-neutral-900'>
                      {client.name}
                    </span>
                    <span className='block truncate font-mono-ui text-[11px] text-neutral-400'>
                      {client.client_id}
                    </span>
                  </span>
                  <Check className={cn('ml-auto', value === client.id ? 'opacity-100' : 'opacity-0')} />
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}
