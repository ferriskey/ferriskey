import { useMemo, useState } from 'react'
import { Search } from 'lucide-react'
import { Checkbox } from '@/components/ui/checkbox'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import {
  WEBHOOK_CATEGORIES,
  WEBHOOK_TRIGGER_COUNT,
} from '../webhook-trigger-catalogue'

import WebhookTrigger = Schemas.WebhookTrigger

export interface WebhookSubscribersFieldProps {
  value: WebhookTrigger[]
  onChange: (next: WebhookTrigger[]) => void
}

export default function WebhookSubscribersField({
  value,
  onChange,
}: WebhookSubscribersFieldProps) {
  const [query, setQuery] = useState('')

  const groups = useMemo(() => {
    const q = query.trim().toLowerCase()
    return WEBHOOK_CATEGORIES.map((category) => ({
      category: category.category,
      total: category.events.length,
      events: q
        ? category.events.filter((event) =>
            `${event.key} ${event.label} ${event.description}`.toLowerCase().includes(q)
          )
        : category.events,
    })).filter((category) => category.events.length > 0)
  }, [query])

  const toggle = (trigger: WebhookTrigger) =>
    onChange(
      value.includes(trigger)
        ? value.filter((x) => x !== trigger)
        : [...value, trigger]
    )

  const selectAll = (keys: WebhookTrigger[]) =>
    onChange([...new Set([...value, ...keys])])

  const clearAll = (keys: WebhookTrigger[]) =>
    onChange(value.filter((v) => !keys.includes(v)))

  return (
    <div className='space-y-3'>
      <div className='flex flex-wrap items-center gap-3'>
        <label className='relative flex h-8 min-w-0 flex-1 items-center sm:max-w-sm'>
          <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400' />
          <input
            type='search'
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder='Filter events…'
            className='h-full w-full rounded-md border border-fk-line pl-8 pr-2 text-xs outline-none placeholder:text-neutral-400 focus:border-fk-primary-border'
          />
        </label>
        <span className='tnum text-xs text-neutral-500'>
          {value.length}/{WEBHOOK_TRIGGER_COUNT} selected
        </span>
        {value.length > 0 && (
          <button
            type='button'
            onClick={() => onChange([])}
            className='cursor-pointer text-xs text-neutral-500 underline-offset-2 hover:text-fk-danger hover:underline'
          >
            Clear all
          </button>
        )}
      </div>

      {groups.length === 0 ? (
        <p className='rounded-lg border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500'>
          No event matches “{query}”.
        </p>
      ) : (
        <div className={cn(tokens.surface.panel, tokens.surface.divider)}>
          {groups.map((group) => {
            const keys = group.events.map((event) => event.key)
            const selected = keys.filter((key) => value.includes(key)).length
            const all = selected === keys.length

            return (
              <section key={group.category} className='px-4 py-3'>
                <div className='flex items-center gap-2 pb-2'>
                  <h3 className='text-xs font-semibold uppercase tracking-wide text-neutral-900'>
                    {group.category}
                  </h3>
                  <span
                    className={cn(
                      'tnum text-xs',
                      selected > 0 ? 'text-fk-primary-text' : 'text-neutral-400'
                    )}
                  >
                    {selected}/{group.total}
                  </span>
                  <span className='flex-1' />
                  <button
                    type='button'
                    onClick={() => selectAll(keys)}
                    disabled={all}
                    className='cursor-pointer text-xs text-neutral-500 underline-offset-2 hover:text-fk-primary-text hover:underline disabled:pointer-events-none disabled:text-neutral-300'
                  >
                    all
                  </button>
                  <span className='text-xs text-neutral-300'>·</span>
                  <button
                    type='button'
                    onClick={() => clearAll(keys)}
                    disabled={selected === 0}
                    className='cursor-pointer text-xs text-neutral-500 underline-offset-2 hover:text-fk-primary-text hover:underline disabled:pointer-events-none disabled:text-neutral-300'
                  >
                    none
                  </button>
                </div>

                <div className='grid gap-x-6 gap-y-1.5 sm:grid-cols-2 xl:grid-cols-3'>
                  {group.events.map((event) => {
                    const on = value.includes(event.key)
                    return (
                      <label
                        key={event.key}
                        htmlFor={event.key}
                        title={event.description}
                        className={cn(
                          'flex min-w-0 cursor-pointer items-start gap-2 rounded-md px-1.5 py-1 transition-colors',
                          on ? 'bg-fk-primary-soft/50' : 'hover:bg-neutral-50'
                        )}
                      >
                        <Checkbox
                          id={event.key}
                          checked={on}
                          onCheckedChange={() => toggle(event.key)}
                          className='mt-0.5'
                        />
                        <span className='min-w-0'>
                          <span
                            className={cn(
                              'block truncate text-xs',
                              on ? 'text-fk-primary-text' : 'text-neutral-700'
                            )}
                          >
                            {event.label}
                          </span>
                          <span className='block truncate font-mono-ui text-xs text-neutral-400'>
                            {event.key}
                          </span>
                        </span>
                      </label>
                    )
                  })}
                </div>
              </section>
            )
          })}
        </div>
      )}
    </div>
  )
}
