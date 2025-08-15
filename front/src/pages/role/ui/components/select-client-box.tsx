import { Button } from '@/components/ui/button'
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from '@/components/ui/command'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { cn } from '@/lib/utils'
import { Check, ChevronsUpDown } from 'lucide-react'
import { useState } from 'react'
import { Schemas } from '@/api/api.client.ts'
import Client = Schemas.Client

export interface SelectClientBoxProps {
  clients: Client[]
  onValueChange: (value: string) => void
}

export default function SelectClientBox({ clients, onValueChange }: SelectClientBoxProps) {
  const [open, setOpen] = useState(false)
  const [selectedClient, setSelectedClient] = useState<Client | null>(null)

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant='outline'
          role='combobox'
          aria-expanded={open}
          className='w-[200px] h-12 justify-between text-neutral-600'
        >
          {selectedClient
            ? clients.find((client) => client.id === selectedClient.id)?.name
            : 'Select client...'}
          <ChevronsUpDown className='opacity-50' />
        </Button>
      </PopoverTrigger>
      <PopoverContent className='w-[300px] p-0' align='start'>
        <Command>
          <CommandInput placeholder='Search client...' className='h-9' />
          <CommandList>
            <CommandEmpty>No client found.</CommandEmpty>
            <CommandGroup>
              {clients.map((client) => (
                <CommandItem
                  key={client.id}
                  value={client.name.toLowerCase()}
                  onSelect={() => {
                    setSelectedClient(client)
                    onValueChange(client.id)
                    setOpen(false)
                  }}
                >
                  {client.name}
                  <Check
                    className={cn(
                      'ml-auto',
                      selectedClient?.id === client.id ? 'opacity-100' : 'opacity-0'
                    )}
                  />
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}
