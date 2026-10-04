import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Check, ChevronsUpDown, Loader2, Plus, Search } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { useUserRealmSearch } from '@/api/realm.api'
import { CreateRealmDialog } from './create-realm-dialog'

export function RealmPicker({
  realm,
  hrefFor,
}: {
  realm: string
  hrefFor: (realmName: string) => string
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [open, setOpen] = useState(false)
  const [createOpen, setCreateOpen] = useState(false)
  const {
    search: query,
    setSearch: setQuery,
    realms: matches,
    isLoading,
  } = useUserRealmSearch({ realm, enabled: open })

  const changeOpen = (next: boolean) => {
    setOpen(next)
    if (!next) setQuery('')
  }

  return (
    <>
      <Popover open={open} onOpenChange={changeOpen}>
        <PopoverTrigger asChild>
          <button
            type='button'
            className='inline-flex shrink-0 cursor-pointer items-center gap-1.5 rounded-md border border-fk-line px-2 py-1 text-[13px] font-medium transition-colors hover:bg-neutral-100 dark:hover:bg-fk-raised'
          >
            {realm.toLowerCase()}
            <ChevronsUpDown className='size-3 text-neutral-400 dark:text-neutral-500' />
          </button>
        </PopoverTrigger>
        <PopoverContent align='start' sideOffset={6} className='w-56 p-0'>
          <label className='flex items-center gap-2 border-b border-fk-line px-2.5'>
            <Search className='size-3.5 shrink-0 text-neutral-400 dark:text-neutral-500' />
            <input
              autoFocus
              type='search'
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t('shell.realm_picker.search_placeholder')}
              className='h-8 w-full bg-transparent text-[13px] outline-none placeholder:text-neutral-400 dark:placeholder:text-neutral-500'
            />
          </label>
          {isLoading ? (
            <div className='flex justify-center px-3 py-4'>
              <Loader2 className='size-4 animate-spin text-neutral-400 dark:text-neutral-500' />
            </div>
          ) : matches.length > 0 ? (
            <ul className='max-h-[calc(7*2rem+0.5rem)] overflow-y-auto p-1'>
              {matches.map((r) => (
                <li key={r.id}>
                  <button
                    type='button'
                    onClick={() => {
                      changeOpen(false)
                      navigate(hrefFor(r.name))
                    }}
                    className='flex h-8 w-full cursor-pointer items-center gap-2 rounded-md px-1.5 text-left transition-colors hover:bg-neutral-100 dark:hover:bg-fk-raised'
                  >
                    <span className='grid size-5 shrink-0 place-items-center rounded bg-fk-primary text-[10px] font-semibold uppercase text-white'>
                      {r.name.charAt(0)}
                    </span>
                    <span className='min-w-0 flex-1 truncate font-mono-ui text-[11px]'>
                      {r.name}
                    </span>
                    {r.name === realm && <Check className='size-3.5 shrink-0 text-fk-success' />}
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <p className='px-3 py-4 text-center text-[13px] text-neutral-500 dark:text-neutral-400'>
              {t('shell.realm_picker.no_match', { query })}
            </p>
          )}
          <div className='border-t border-fk-line p-1'>
            <button
              type='button'
              onClick={() => {
                changeOpen(false)
                setCreateOpen(true)
              }}
              className='flex h-8 w-full cursor-pointer items-center gap-2 rounded-md px-1.5 text-left text-[13px] transition-colors hover:bg-neutral-100 dark:hover:bg-fk-raised'
            >
              <span className='grid size-5 shrink-0 place-items-center rounded border border-fk-line'>
                <Plus className='size-3' />
              </span>
              {t('shell.realm_picker.create')}
            </button>
          </div>
        </PopoverContent>
      </Popover>

      <CreateRealmDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        realm={realm}
        onCreated={(realmName) => navigate(hrefFor(realmName))}
      />
    </>
  )
}
