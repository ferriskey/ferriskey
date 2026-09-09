import { useMemo, useState, type ReactNode } from 'react'
import { Link } from 'react-router-dom'
import { KeyRound, Plus, Search, Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { IconTile, Pill, Section, Segmented } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import ClientScope = Schemas.ClientScope
import ScopeType = Schemas.ScopeType

export type AssignableScopeType = 'default' | 'optional'

export interface ClientScopesTabProps {
  view: string
  onViewChange: (view: string) => void
  assignedScopes: ClientScope[]
  availableScopes: ClientScope[]
  isLoading: boolean
  isLoadingAvailable: boolean
  isAssigning: boolean
  addOpen: boolean
  scopeHref: (scope: ClientScope) => string
  onAddOpenChange: (open: boolean) => void
  onAdd: (scopeId: string, type: AssignableScopeType) => void
  onSetType: (scope: ClientScope, type: AssignableScopeType) => void
  onRemove: (scope: ClientScope) => void
  evaluatePanel: ReactNode
}

const assignableType = (scope: ClientScope): AssignableScopeType | null => {
  if (scope.default_scope_type === 'DEFAULT') return 'default'
  if (scope.default_scope_type === 'OPTIONAL') return 'optional'
  return null
}

const scopeToneOf = (type: ScopeType) =>
  type === 'DEFAULT' ? 'success' : type === 'OPTIONAL' ? 'info' : 'amber'

function AddScopeDialog({
  open,
  onOpenChange,
  available,
  isLoading,
  isAssigning,
  onAdd,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  available: ClientScope[]
  isLoading: boolean
  isAssigning: boolean
  onAdd: (scopeId: string, type: AssignableScopeType) => void
}) {
  const [query, setQuery] = useState('')
  const [type, setType] = useState<AssignableScopeType>('default')
  const [selected, setSelected] = useState<string | null>(null)

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase()
    if (!q) return available
    return available.filter((s) => `${s.name} ${s.description ?? ''}`.toLowerCase().includes(q))
  }, [available, query])

  const close = () => {
    setQuery('')
    setSelected(null)
    setType('default')
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={(next) => (next ? onOpenChange(next) : close())}>
      <DialogContent className='sm:max-w-lg'>
        <DialogHeader>
          <DialogTitle>Add a client scope</DialogTitle>
          <DialogDescription>
            A default scope goes into every token; an optional one only arrives when the client
            asks for it.
          </DialogDescription>
        </DialogHeader>

        <div className='flex items-center gap-2'>
          <label className='relative flex h-8 min-w-0 flex-1 items-center'>
            <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400' />
            <input
              type='search'
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder='Search scopes…'
              className='h-full w-full rounded-md border border-fk-line pl-8 pr-2 text-xs outline-none placeholder:text-neutral-400 focus:border-fk-primary-border'
            />
          </label>
          <Select value={type} onValueChange={(v) => setType(v as AssignableScopeType)}>
            <SelectTrigger className='h-8 w-36 text-xs'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value='default'>Default</SelectItem>
              <SelectItem value='optional'>Optional</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className='max-h-72 overflow-y-auto rounded-md border border-fk-line'>
          {isLoading ? (
            <p className='px-3 py-6 text-center text-xs text-neutral-500'>
              Loading available scopes…
            </p>
          ) : rows.length > 0 ? (
            <ul className='divide-y divide-fk-line-soft'>
              {rows.map((scope) => (
                <li key={scope.id}>
                  <label
                    htmlFor={`add-scope-${scope.id}`}
                    className={cn(
                      'flex cursor-pointer items-start gap-2.5 px-3 py-2 transition-colors',
                      selected === scope.id ? 'bg-fk-primary-soft/50' : 'hover:bg-neutral-50'
                    )}
                  >
                    <Checkbox
                      id={`add-scope-${scope.id}`}
                      checked={selected === scope.id}
                      onCheckedChange={() => setSelected(scope.id)}
                      className='mt-0.5'
                    />
                    <span className='min-w-0'>
                      <span className='flex flex-wrap items-center gap-2'>
                        <span className='font-mono-ui text-xs text-neutral-900'>{scope.name}</span>
                        <Pill mono>{scope.protocol}</Pill>
                      </span>
                      <span className='mt-0.5 block text-xs text-neutral-500'>
                        {scope.description || 'No description'}
                      </span>
                    </span>
                  </label>
                </li>
              ))}
            </ul>
          ) : (
            <p className='px-3 py-6 text-center text-xs text-neutral-500'>
              {query
                ? `No scope matches “${query}”.`
                : 'Every scope of the realm is already assigned.'}
            </p>
          )}
        </div>

        <DialogFooter>
          <Button variant='ghost' onClick={close} disabled={isAssigning}>
            Cancel
          </Button>
          <Button
            disabled={!selected || isAssigning}
            onClick={() => {
              if (selected) onAdd(selected, type)
              close()
            }}
          >
            {isAssigning ? 'Assigning…' : 'Assign scope'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default function ClientScopesTab({
  view,
  onViewChange,
  assignedScopes,
  availableScopes,
  isLoading,
  isLoadingAvailable,
  isAssigning,
  addOpen,
  scopeHref,
  onAddOpenChange,
  onAdd,
  onSetType,
  onRemove,
  evaluatePanel,
}: ClientScopesTabProps) {
  const defaults = assignedScopes.filter((s) => s.default_scope_type === 'DEFAULT')

  return (
    <>
      <Segmented
        value={view}
        onChange={onViewChange}
        items={[
          { key: 'assigned', label: 'Assigned scopes' },
          { key: 'evaluate', label: 'Evaluate' },
        ]}
      />

      {view === 'evaluate' ? (
        evaluatePanel
      ) : (
        <div className={tokens.page.blockGap}>
          <Section
            title={`Assigned scopes (${assignedScopes.length})`}
            description='Sets of claims this client is allowed to request.'
            action={
              <Button
                size='sm'
                disabled={availableScopes.length === 0}
                onClick={() => onAddOpenChange(true)}
              >
                <Plus /> Add scope
              </Button>
            }
            contained={assignedScopes.length > 0}
          >
            {isLoading ? (
              <div className={cn(tokens.surface.panel, tokens.surface.divider)}>
                {Array.from({ length: 3 }).map((_, i) => (
                  <div key={i} className='flex items-center gap-3 px-3 py-3'>
                    <div className='size-7 animate-pulse rounded-md bg-neutral-100' />
                    <div className='h-3 w-40 animate-pulse rounded bg-neutral-100' />
                  </div>
                ))}
              </div>
            ) : assignedScopes.length > 0 ? (
              <ul className={tokens.surface.divider}>
                {assignedScopes.map((scope) => {
                  const type = assignableType(scope)

                  return (
                    <li key={scope.id} className='flex items-center gap-3 py-2.5'>
                      <IconTile tone={scopeToneOf(scope.default_scope_type)} className='size-7'>
                        <KeyRound className='size-3.5' strokeWidth={1.75} />
                      </IconTile>

                      <div className='min-w-0 flex-1'>
                        <div className='flex flex-wrap items-center gap-2'>
                          <Link
                            to={scopeHref(scope)}
                            className='font-mono-ui text-xs text-neutral-900 underline-offset-2 hover:text-fk-primary-text hover:underline'
                          >
                            {scope.name}
                          </Link>
                          <Pill mono>{scope.protocol}</Pill>
                        </div>
                        <p className='mt-0.5 truncate text-xs text-neutral-500'>
                          {scope.description || 'No description'}
                        </p>
                      </div>

                      {type ? (
                        <div className='flex shrink-0 overflow-hidden rounded-md border border-fk-line'>
                          {(['default', 'optional'] as const).map((candidate) => (
                            <button
                              key={candidate}
                              type='button'
                              aria-pressed={type === candidate}
                              onClick={() => type !== candidate && onSetType(scope, candidate)}
                              className={cn(
                                'cursor-pointer px-2 py-1 text-xs transition-colors',
                                type === candidate
                                  ? 'bg-fk-primary-soft font-medium text-fk-primary-text'
                                  : 'text-neutral-500 hover:bg-neutral-50'
                              )}
                            >
                              {candidate}
                            </button>
                          ))}
                        </div>
                      ) : (
                        <Pill tone='amber' mono>
                          none
                        </Pill>
                      )}

                      <Button
                        variant='ghost'
                        size='icon'
                        disabled={!type}
                        aria-label={`Remove ${scope.name}`}
                        onClick={() => onRemove(scope)}
                        className='size-7 text-neutral-400 hover:text-fk-danger'
                      >
                        <Trash2 />
                      </Button>
                    </li>
                  )
                })}
              </ul>
            ) : (
              <p className='rounded-lg border border-dashed border-fk-amber-border bg-fk-amber-soft/40 px-4 py-3 text-xs text-neutral-600'>
                No scope assigned — the tokens of this client will carry no profile claim.
              </p>
            )}
          </Section>

          <Section
            title='Effective scopes'
            description='Included in every token issued for this client, without the client having to ask.'
            contained={defaults.length > 0}
          >
            {defaults.length > 0 ? (
              <ul className={tokens.surface.divider}>
                {defaults.map((scope) => (
                  <li key={scope.id} className='py-2.5'>
                    <div className='flex flex-wrap items-center gap-2'>
                      <Pill tone='success' mono>
                        {scope.name}
                      </Pill>
                      <span className='text-xs text-neutral-500'>
                        {scope.description || 'No description'}
                      </span>
                    </div>
                    {scope.protocol_mappers && scope.protocol_mappers.length > 0 ? (
                      <div className='mt-1.5 flex flex-wrap gap-1'>
                        {scope.protocol_mappers.map((mapper) => (
                          <code
                            key={mapper.id}
                            title={mapper.mapper_type}
                            className='rounded border border-fk-line bg-neutral-50 px-1.5 py-0.5 font-mono-ui text-[11px] text-neutral-600'
                          >
                            {mapper.name}
                          </code>
                        ))}
                      </div>
                    ) : (
                      <p className='mt-1 text-xs text-neutral-400'>
                        No mapper: this scope produces no claim, it only opens a right.
                      </p>
                    )}
                  </li>
                ))}
              </ul>
            ) : (
              <p className='rounded-lg border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500'>
                No default scope: every claim will have to be requested explicitly by the client.
              </p>
            )}
          </Section>
        </div>
      )}

      <AddScopeDialog
        open={addOpen}
        onOpenChange={onAddOpenChange}
        available={availableScopes}
        isLoading={isLoadingAvailable}
        isAssigning={isAssigning}
        onAdd={onAdd}
      />
    </>
  )
}
