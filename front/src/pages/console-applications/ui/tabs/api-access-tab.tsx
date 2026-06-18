import { Schemas } from '@/api/api.client'
import { useGetClientScopes, useAssignScope, useUnassignScope } from '@/api/client.api'
import { useGetClientScopes as useGetRealmScopes } from '@/api/client-scope.api'
import MultipleSelector, { type Option } from '@/components/ui/multiselect'
import { KeyRound, Loader2, Plus, X } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Section } from './primitives'

type ScopeKind = 'default' | 'optional'

function scopeKind(s: Schemas.ClientScope): ScopeKind {
  return s.default_scope_type === 'DEFAULT' ? 'default' : 'optional'
}

interface Props {
  realm: string
  clientId: string
}

export default function ApiAccessTab({ realm, clientId }: Props) {
  const { data: assignedRaw, isLoading } = useGetClientScopes({ realm, clientId })
  const { data: realmScopes } = useGetRealmScopes({ realm })
  const assignScope = useAssignScope()
  const unassignScope = useUnassignScope()

  // The assigned-scopes endpoint may return a bare array or a {data} envelope.
  const assigned: Schemas.ClientScope[] = useMemo(() => {
    if (Array.isArray(assignedRaw)) return assignedRaw
    const maybe = assignedRaw as { data?: Schemas.ClientScope[] } | undefined
    return maybe?.data ?? []
  }, [assignedRaw])

  const assignedIds = useMemo(() => new Set(assigned.map((s) => s.id)), [assigned])
  const available = useMemo(
    () => (realmScopes?.data ?? []).filter((s) => !assignedIds.has(s.id)),
    [realmScopes, assignedIds],
  )

  const [pendingId, setPendingId] = useState<string | null>(null)
  const [picked, setPicked] = useState<Option[]>([])
  const [pickKind, setPickKind] = useState<ScopeKind>('default')
  const [adding, setAdding] = useState(false)

  const defaults = assigned.filter((s) => scopeKind(s) === 'default')
  const optionals = assigned.filter((s) => scopeKind(s) === 'optional')

  const options: Option[] = useMemo(
    () => available.map((s) => ({ value: s.id, label: s.name })),
    [available],
  )

  async function handleAdd() {
    if (picked.length === 0) return
    setAdding(true)
    try {
      // Assign sequentially so a mid-list failure surfaces a toast but the
      // earlier scopes still land.
      for (const opt of picked) {
        await assignScope.mutateAsync({ realm, clientId, scopeId: opt.value, type: pickKind })
      }
      setPicked([])
    } finally {
      setAdding(false)
    }
  }

  async function handleChangeKind(scopeId: string, from: ScopeKind) {
    const to: ScopeKind = from === 'default' ? 'optional' : 'default'
    setPendingId(scopeId)
    try {
      await unassignScope.mutateAsync({ realm, clientId, scopeId, type: from })
      await assignScope.mutateAsync({ realm, clientId, scopeId, type: to })
    } finally {
      setPendingId(null)
    }
  }

  async function handleRemove(scopeId: string, kind: ScopeKind) {
    setPendingId(scopeId)
    try {
      await unassignScope.mutateAsync({ realm, clientId, scopeId, type: kind })
    } finally {
      setPendingId(null)
    }
  }

  return (
    <div className='flex flex-col gap-6'>
      <Section
        title='Client scopes'
        description='Scopes decide which claims and permissions land in the tokens issued to this application. Default scopes are always granted; optional scopes must be requested via the scope parameter.'
      >
        {/* Add composer — pick one or many scopes, assign them all as the chosen kind */}
        <div className='flex items-start gap-2'>
          <MultipleSelector
            className='flex-1'
            value={picked}
            onChange={setPicked}
            options={options}
            disabled={available.length === 0 || adding}
            hidePlaceholderWhenSelected
            placeholder={
              available.length === 0 ? 'No more scopes available' : 'Select scopes to add…'
            }
            emptyIndicator={
              <p className='text-center text-xs text-muted-foreground py-2'>No matching scopes.</p>
            }
          />
          <Segmented value={pickKind} onChange={setPickKind} />
          <button
            type='button'
            onClick={handleAdd}
            disabled={picked.length === 0 || adding}
            className='inline-flex items-center gap-1.5 rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed shrink-0'
          >
            {adding ? <Loader2 className='h-3.5 w-3.5 animate-spin' /> : <Plus className='h-3.5 w-3.5' />}
            Add{picked.length > 0 ? ` (${picked.length})` : ''}
          </button>
        </div>

        {/* Lists */}
        {isLoading ? (
          <p className='text-xs text-muted-foreground'>Loading scopes…</p>
        ) : assigned.length === 0 ? (
          <div className='rounded-md border border-dashed border-border bg-muted/20 p-6 text-center'>
            <KeyRound className='mx-auto h-5 w-5 text-muted-foreground' />
            <p className='text-sm font-medium mt-2'>No scopes assigned</p>
            <p className='text-xs text-muted-foreground mt-0.5'>
              Add a scope above to control what this application can request.
            </p>
          </div>
        ) : (
          <div className='flex flex-col gap-5'>
            <ScopeGroup
              label='Default'
              hint='Always granted'
              scopes={defaults}
              pendingId={pendingId}
              onChangeKind={handleChangeKind}
              onRemove={handleRemove}
            />
            <ScopeGroup
              label='Optional'
              hint='Granted only when requested'
              scopes={optionals}
              pendingId={pendingId}
              onChangeKind={handleChangeKind}
              onRemove={handleRemove}
            />
          </div>
        )}
      </Section>

      {/* Token preview */}
      <Section
        title='Token preview'
        description='Scopes included in a token issued without an explicit scope request.'
      >
        {defaults.length === 0 ? (
          <p className='text-xs text-muted-foreground'>
            No default scopes — tokens carry only the base claims.
          </p>
        ) : (
          <div className='flex flex-wrap gap-1.5'>
            {defaults.map((s) => (
              <span
                key={s.id}
                className='inline-flex items-center rounded-md bg-primary/10 px-2 py-0.5 text-xs font-mono text-primary'
              >
                {s.name}
              </span>
            ))}
          </div>
        )}
      </Section>
    </div>
  )
}

interface ScopeGroupProps {
  label: string
  hint: string
  scopes: Schemas.ClientScope[]
  pendingId: string | null
  onChangeKind: (scopeId: string, from: ScopeKind) => void
  onRemove: (scopeId: string, kind: ScopeKind) => void
}

function ScopeGroup({ label, hint, scopes, pendingId, onChangeKind, onRemove }: ScopeGroupProps) {
  if (scopes.length === 0) return null
  return (
    <div className='flex flex-col gap-2'>
      <div className='flex items-center gap-2'>
        <h3 className='text-xs font-semibold uppercase tracking-wide text-muted-foreground'>{label}</h3>
        <span className='text-[10px] text-muted-foreground/70'>· {hint}</span>
      </div>
      <div className='flex flex-col divide-y divide-border rounded-md border border-border'>
        {scopes.map((scope) => {
          const kind = scopeKind(scope)
          const busy = pendingId === scope.id
          return (
            <div key={scope.id} className='flex items-center gap-3 px-3 py-2.5'>
              <div className='flex-1 min-w-0'>
                <div className='flex items-center gap-2'>
                  <span className='text-sm font-medium truncate'>{scope.name}</span>
                  <span className='rounded bg-muted px-1 py-0.5 text-[10px] font-mono text-muted-foreground'>
                    {scope.protocol}
                  </span>
                </div>
                <p className='text-xs text-muted-foreground truncate'>
                  {scope.description || 'No description'}
                </p>
              </div>
              {busy ? (
                <Loader2 className='h-4 w-4 animate-spin text-muted-foreground' />
              ) : (
                <>
                  <Segmented
                    value={kind}
                    onChange={(next) => next !== kind && onChangeKind(scope.id, kind)}
                  />
                  <button
                    type='button'
                    onClick={() => onRemove(scope.id, kind)}
                    className='inline-flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:text-red-500 hover:bg-muted transition-colors shrink-0'
                    aria-label={`Remove ${scope.name}`}
                  >
                    <X className='h-3.5 w-3.5' />
                  </button>
                </>
              )}
            </div>
          )
        })}
      </div>
    </div>
  )
}

function Segmented({ value, onChange }: { value: ScopeKind; onChange: (v: ScopeKind) => void }) {
  const opts: { key: ScopeKind; label: string }[] = [
    { key: 'default', label: 'Default' },
    { key: 'optional', label: 'Optional' },
  ]
  return (
    <div className='inline-flex rounded-md border border-border bg-muted/40 p-0.5'>
      {opts.map((o) => (
        <button
          key={o.key}
          type='button'
          onClick={() => onChange(o.key)}
          className={`rounded px-2 py-1 text-xs font-medium transition-colors ${
            value === o.key
              ? 'bg-background text-foreground shadow-sm'
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          {o.label}
        </button>
      ))}
    </div>
  )
}
