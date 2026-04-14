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
  /** Map from item ID to whitelist entry ID (for removal) */
  entryIdMap: Record<string, string>
  /** Entries inherited from realm (read-only) */
  inheritedEntries?: InheritedEntry[]
  placeholder?: string
  emptyMessage?: string
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

  const hasEntries = whitelistedItems.length > 0 || inheritedEntries.length > 0

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
                      <div className='flex flex-col'>
                        <span className='text-sm'>{item.label}</span>
                        {item.sublabel && (
                          <span className='text-xs text-muted-foreground'>{item.sublabel}</span>
                        )}
                      </div>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </CommandList>
            </Command>
          </PopoverContent>
        </Popover>
      </div>

      {hasEntries ? (
        <div className='rounded-md border'>
          <table className='w-full text-sm'>
            <thead>
              <tr className='border-b bg-muted/50'>
                <th className='px-4 py-2 text-left font-medium'>Name</th>
                <th className='px-4 py-2 text-left font-medium'>Source</th>
                <th className='px-4 py-2 text-right font-medium'>Actions</th>
              </tr>
            </thead>
            <tbody>
              {inheritedEntries.map((entry) => (
                <tr key={`realm-${entry.id}`} className='border-b last:border-0'>
                  <td className='px-4 py-2'>
                    <div className='flex flex-col'>
                      <span>{entry.label}</span>
                      {entry.sublabel && (
                        <span className='text-xs text-muted-foreground'>{entry.sublabel}</span>
                      )}
                    </div>
                  </td>
                  <td className='px-4 py-2'>
                    <span className='inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-muted text-muted-foreground border'>
                      Realm
                    </span>
                  </td>
                  <td className='px-4 py-2 text-right' />
                </tr>
              ))}
              {whitelistedItems.map((item) => (
                <tr key={item.id} className='border-b last:border-0'>
                  <td className='px-4 py-2'>
                    <div className='flex flex-col'>
                      <span>{item.label}</span>
                      {item.sublabel && (
                        <span className='text-xs text-muted-foreground'>{item.sublabel}</span>
                      )}
                    </div>
                  </td>
                  <td className='px-4 py-2'>
                    <span className='inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-primary/10 text-primary border border-primary/30'>
                      Client
                    </span>
                  </td>
                  <td className='px-4 py-2 text-right'>
                    <Button
                      type='button'
                      variant='ghost'
                      size='sm'
                      onClick={() => onRemove(entryIdMap[item.id])}
                    >
                      <Trash2 className='h-4 w-4 text-destructive' />
                    </Button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p className='text-sm text-muted-foreground text-center py-2'>No entries configured.</p>
      )}
    </div>
  )
}
