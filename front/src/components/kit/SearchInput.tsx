import { Search } from 'lucide-react'
import { cn } from '@/lib/utils'

export function SearchInput({
  value,
  onChange,
  placeholder,
  className,
}: {
  value: string
  onChange: (value: string) => void
  placeholder: string
  className?: string
}) {
  return (
    <label className={cn('relative flex h-8 min-w-[15rem] flex-1 items-center', className)}>
      <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400 dark:text-neutral-500' />
      <input
        type='search'
        aria-label={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className='h-full w-full rounded-md border border-fk-line bg-white pl-8 pr-3 text-sm outline-none placeholder:text-neutral-400 focus:border-fk-primary-border focus:ring-2 focus:ring-fk-primary/15 dark:bg-fk-surface'
      />
    </label>
  )
}
