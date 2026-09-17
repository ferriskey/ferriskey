import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Check, ChevronsUpDown, Languages, Search } from 'lucide-react'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { useLocale } from '@/hooks/use-locale'

export function LanguageSwitcher() {
  const { t } = useTranslation()
  const { locale, locales, setLocale } = useLocale()
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')

  const options = useMemo(
    () => locales.map((value) => ({ value, name: t(`language.names.${value}`, { lng: value }) })),
    [locales, t]
  )

  const matches = useMemo(() => {
    const q = query.trim().toLowerCase()
    if (!q) return options
    return options.filter(
      (option) => option.name.toLowerCase().includes(q) || option.value.toLowerCase().includes(q)
    )
  }, [options, query])

  const activeName = options.find((option) => option.value === locale)?.name ?? locale

  return (
    <Popover
      open={open}
      onOpenChange={(next) => {
        setOpen(next)
        if (!next) setQuery('')
      }}
    >
      <PopoverTrigger asChild>
        <button
          type='button'
          aria-label={t('language.switcher_label')}
          className='inline-flex shrink-0 cursor-pointer items-center gap-1.5 rounded-md border border-fk-line px-2 py-1 text-[13px] font-medium transition-colors hover:bg-neutral-100 dark:hover:bg-fk-raised'
        >
          <Languages className='size-3.5 shrink-0 text-neutral-400 dark:text-neutral-500' />
          <span lang={locale}>{activeName}</span>
          <ChevronsUpDown className='size-3 text-neutral-400 dark:text-neutral-500' />
        </button>
      </PopoverTrigger>
      <PopoverContent align='end' sideOffset={6} className='w-56 p-0'>
        <label className='flex items-center gap-2 border-b border-fk-line px-2.5'>
          <Search className='size-3.5 shrink-0 text-neutral-400 dark:text-neutral-500' />
          <input
            autoFocus
            type='search'
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t('language.search_placeholder')}
            className='h-8 w-full bg-transparent text-[13px] outline-none placeholder:text-neutral-400 dark:placeholder:text-neutral-500'
          />
        </label>
        {matches.length > 0 ? (
          <ul className='max-h-[calc(7*2rem+0.5rem)] overflow-y-auto p-1'>
            {matches.map((option) => (
              <li key={option.value}>
                <button
                  type='button'
                  lang={option.value}
                  onClick={() => {
                    setOpen(false)
                    void setLocale(option.value)
                  }}
                  className='flex h-8 w-full cursor-pointer items-center gap-2 rounded-md px-1.5 text-left transition-colors hover:bg-neutral-100 dark:hover:bg-fk-raised'
                >
                  <span className='min-w-0 flex-1 truncate text-[13px]'>{option.name}</span>
                  <span className='shrink-0 font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
                    {option.value}
                  </span>
                  {option.value === locale && (
                    <Check className='size-3.5 shrink-0 text-fk-success' />
                  )}
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <p className='px-3 py-4 text-center text-[13px] text-neutral-500 dark:text-neutral-400'>
            {t('language.no_match', { query })}
          </p>
        )}
      </PopoverContent>
    </Popover>
  )
}
