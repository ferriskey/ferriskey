import { useState } from 'react'
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
import { ChevronsUpDown, Plus, Trash2 } from 'lucide-react'

interface WhitelistItem {
  id: string
  label: string
  sublabel?: string
}

interface InheritedEntry {
  id: string
  label: string
  sublabel?: string
}

interface WhitelistPickerProps {
  title: string
  description: string
  items: WhitelistItem[]
  whitelistedIds: string[]
  onAdd: (id: string) => void
  onRemove: (entryId: string) => void
  entryIdMap: Record<string, string>
  inheritedEntries?: InheritedEntry[]
  placeholder?: string
  emptyMessage?: string
}

function ItemAvatar({ label }: { label: string }) {
  return (
    <div className='h-10 w-10 shrink-0 rounded-lg bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center border border-primary/20'>
      <span className='text-sm font-semibold text-primary'>
        {label[0]?.toUpperCase() || '?'}
      </span>
    </div>
  )
}

export default function WhitelistPicker({
  title,
  description,
  items,
  whitelistedIds,
  onAdd,
  onRemove,
  entryIdMap,
  inheritedEntries = [],
  placeholder = 'Search...',
  emptyMessage = 'No items found.',
}: WhitelistPickerProps) {
  const [open, setOpen] = useState(false)

  const allVisibleIds = [...whitelistedIds, ...inheritedEntries.map((e) => e.id)]
  const availableItems = items.filter((item) => !allVisibleIds.includes(item.id))
  const whitelistedItems = items.filter((item) => whitelistedIds.includes(item.id))

  const allEntries = [
    ...inheritedEntries.map((e) => ({ ...e, source: 'realm' as const })),
    ...whitelistedItems.map((e) => ({ ...e, source: 'client' as const })),
  ]

  return (
    <div className='flex flex-col gap-4 py-4 border-t'>
      <div className='flex items-center justify-between'>
        <div>
          <p className='text-sm font-medium'>{title}</p>
          <p className='text-sm text-muted-foreground mt-0.5'>{description}</p>
        </div>
        <Popover open={open} onOpenChange={setOpen}>
          <PopoverTrigger asChild>
            <Button variant='outline' size='sm' className='gap-1.5'>
              <Plus className='h-3.5 w-3.5' />
              Add
              <ChevronsUpDown className='h-3.5 w-3.5 opacity-50' />
            </Button>
          </PopoverTrigger>
          <PopoverContent className='w-[300px] p-0' align='end'>
            <Command>
              <CommandInput placeholder={placeholder} className='h-9' />
              <CommandList>
                <CommandEmpty>{emptyMessage}</CommandEmpty>
                <CommandGroup>
                  {availableItems.map((item) => (
                    <CommandItem
                      key={item.id}
                      value={item.label}
                      onSelect={() => {
                        onAdd(item.id)
                        setOpen(false)
                      }}
                    >
                      <div className='flex items-center gap-2'>
                        <div className='h-6 w-6 shrink-0 rounded bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center'>
                          <span className='text-xs font-semibold text-primary'>
                            {item.label[0]?.toUpperCase() || '?'}
                          </span>
                        </div>
                        <div className='flex flex-col'>
                          <span className='text-sm'>{item.label}</span>
                          {item.sublabel && (
                            <span className='text-xs text-muted-foreground'>{item.sublabel}</span>
                          )}
                        </div>
                      </div>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </CommandList>
            </Command>
          </PopoverContent>
        </Popover>
      </div>

      {allEntries.length > 0 ? (
        <div className='flex flex-col divide-y'>
          {allEntries.map((entry) => (
            <div key={`${entry.source}-${entry.id}`} className='flex items-center justify-between py-3'>
              <div className='flex items-center gap-3'>
                <ItemAvatar label={entry.label} />
                <div className='flex flex-col'>
                  <div className='flex items-center gap-2'>
                    <span className='text-sm font-semibold'>{entry.label}</span>
                    <span
                      className={`inline-flex items-center px-1.5 py-px rounded text-[9px] font-medium uppercase tracking-wide border ${
                        entry.source === 'realm'
                          ? 'border-border text-muted-foreground bg-muted'
                          : 'border-primary/30 text-primary bg-primary/10'
                      }`}
                    >
                      {entry.source === 'realm' ? 'realm' : 'client'}
                    </span>
                  </div>
                  {entry.sublabel && (
                    <span className='text-xs text-muted-foreground'>{entry.sublabel}</span>
                  )}
                </div>
              </div>
              {entry.source === 'client' && (
                <Button
                  type='button'
                  variant='ghost'
                  size='sm'
                  onClick={() => onRemove(entryIdMap[entry.id])}
                >
                  <Trash2 className='h-4 w-4 text-destructive' />
                </Button>
              )}
            </div>
          ))}
        </div>
      ) : (
        <p className='text-sm text-muted-foreground text-center py-2'>No entries configured.</p>
      )}
    </div>
  )
}
