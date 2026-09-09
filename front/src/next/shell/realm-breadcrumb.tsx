import { useMemo, useState, type ReactNode } from 'react'
import { Check, ChevronRight, ChevronsUpDown, Search } from 'lucide-react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { cn } from '@/lib/utils'
import useRealmStore from '@/store/realm.store'
import { RouterParams } from '@/routes/router'
import { NEXT_URL } from '../routes'
import type { Crumb } from './use-crumbs'

export function RealmBreadcrumb({
  crumbs,
  actions,
}: {
  crumbs: Crumb[]
  actions?: ReactNode
}) {
  const { realm_name = 'master' } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { userRealms } = useRealmStore()
  const [query, setQuery] = useState('')
  const [open, setOpen] = useState(false)

  const matches = useMemo(() => {
    const q = query.trim().toLowerCase()
    if (!q) return userRealms
    return userRealms.filter((r) => r.name.toLowerCase().includes(q))
  }, [query, userRealms])

  return (
    <nav
      aria-label='Breadcrumb'
      className='flex h-14 shrink-0 items-center gap-2 border-b border-fk-line bg-white pl-3 pr-4'
    >
      <img
        src='/logo_ferriskey.png'
        alt='FerrisKey'
        className='size-8 shrink-0 rounded-md'
      />

      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <button
            type='button'
            className='group flex cursor-pointer items-center gap-1.5 rounded-md px-2 py-1.5 text-sm font-medium text-neutral-900 transition-colors hover:bg-neutral-100'
          >
            {realm_name}
            <ChevronsUpDown className='size-3.5 text-neutral-400 transition-colors group-hover:text-neutral-600' />
          </button>
        </PopoverTrigger>

        <PopoverContent align='start' sideOffset={6} className='w-64 p-0'>
          <label className='flex items-center gap-2 border-b border-fk-line px-2.5'>
            <Search className='size-3.5 shrink-0 text-neutral-400' />
            <input
              autoFocus
              type='search'
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder='Search a realm…'
              className='h-8 w-full bg-transparent text-[13px] outline-none placeholder:text-neutral-400'
            />
          </label>

          {matches.length > 0 ? (
            <ul className='max-h-[calc(7*2rem+0.5rem)] overflow-y-auto p-1'>
              {matches.map((r) => (
                <li key={r.id}>
                  <button
                    type='button'
                    onClick={() => {
                      setOpen(false)
                      navigate(`${NEXT_URL(r.name)}/roles`)
                    }}
                    className='flex h-8 w-full cursor-pointer items-center gap-2 rounded-md px-1.5 text-left transition-colors hover:bg-neutral-100'
                  >
                    <span className='grid size-5 shrink-0 place-items-center rounded bg-fk-primary text-[10px] font-semibold uppercase text-white'>
                      {r.name.charAt(0)}
                    </span>
                    <span className='min-w-0 flex-1 truncate font-mono-ui text-[11px] text-neutral-900'>
                      {r.name}
                    </span>
                    {r.name === realm_name && (
                      <Check className='size-3.5 shrink-0 text-fk-success' />
                    )}
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <p className='px-3 py-4 text-center text-[13px] text-neutral-500'>
              No realm for “{query}”
            </p>
          )}
        </PopoverContent>
      </Popover>

      {crumbs.map((crumb, i) => {
        const last = i === crumbs.length - 1
        return (
          <span key={`${crumb.label}-${i}`} className='flex items-center gap-2'>
            <ChevronRight className='size-3.5 shrink-0 text-neutral-300' />
            {crumb.to && !last ? (
              <Link
                to={crumb.to}
                className='rounded px-1 text-sm text-neutral-500 transition-colors hover:text-neutral-900'
              >
                {crumb.label}
              </Link>
            ) : (
              <span
                className={cn(
                  'px-1 text-sm',
                  last ? 'font-medium text-neutral-900' : 'text-neutral-500'
                )}
              >
                {crumb.label}
              </span>
            )}
          </span>
        )
      })}

      {actions && <div className='ml-auto flex items-center gap-2'>{actions}</div>}
    </nav>
  )
}
