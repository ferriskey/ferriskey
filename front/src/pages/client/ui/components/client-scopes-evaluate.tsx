import { useMemo, useState } from 'react'
import { Schemas } from '@/api/api.client'
import { usePreviewToken } from '@/api/client.api'
import { useGetUsers } from '@/api/user.api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible'
import { ChevronDown } from 'lucide-react'

export interface ClientScopesEvaluateProps {
  realm?: string
  clientId?: string
  assignedScopes: Schemas.ClientScope[]
}

function JsonPanel({ title, value }: { title: string; value: unknown }) {
  if (value === undefined || value === null) return null
  return (
    <Collapsible defaultOpen className='border rounded-md'>
      <CollapsibleTrigger className='flex w-full items-center justify-between px-4 py-3 text-sm font-medium'>
        {title}
        <ChevronDown className='h-4 w-4 text-muted-foreground' />
      </CollapsibleTrigger>
      <CollapsibleContent>
        <pre className='overflow-x-auto border-t bg-muted/40 px-4 py-3 text-xs'>
          {JSON.stringify(value, null, 2)}
        </pre>
      </CollapsibleContent>
    </Collapsible>
  )
}

export default function ClientScopesEvaluate({
  realm,
  clientId,
  assignedScopes,
}: ClientScopesEvaluateProps) {
  const { data: usersData } = useGetUsers({ realm })
  const users = usersData?.data ?? []

  const optionalScopes = useMemo(
    () => assignedScopes.filter((s) => s.default_scope_type === 'OPTIONAL'),
    [assignedScopes]
  )
  const defaultScopeNames = useMemo(
    () =>
      assignedScopes
        .filter((s) => s.default_scope_type === 'DEFAULT')
        .map((s) => s.name),
    [assignedScopes]
  )

  const [userId, setUserId] = useState<string | undefined>(undefined)
  const [selectedOptional, setSelectedOptional] = useState<string[]>([])

  const preview = usePreviewToken()

  const toggleOptional = (name: string) => {
    setSelectedOptional((prev) =>
      prev.includes(name) ? prev.filter((n) => n !== name) : [...prev, name]
    )
  }

  const handlePreview = () => {
    if (!realm || !clientId) return
    // Default scopes always apply on the backend; include them in the requested scope string
    // so the `scope` claim is realistic, plus the optional scopes the admin selected.
    const scope = [...new Set([...defaultScopeNames, ...selectedOptional])].join(' ')
    preview.mutate({ realm, clientId, userId, scope: scope || undefined })
  }

  const result = preview.data

  return (
    <div className='flex flex-col gap-6'>
      <div className='flex flex-col gap-4 rounded-md border p-4'>
        <div className='flex flex-col gap-2'>
          <label className='text-sm font-medium'>User (optional)</label>
          <Select value={userId ?? ''} onValueChange={(v) => setUserId(v === 'none' ? undefined : v)}>
            <SelectTrigger className='max-w-md'>
              <SelectValue placeholder='No user — use placeholder values' />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value='none'>No user (placeholder values)</SelectItem>
              {users.map((user) => (
                <SelectItem key={user.id} value={user.id}>
                  {user.username}
                  {user.email ? ` (${user.email})` : ''}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <p className='text-xs text-muted-foreground'>
            Without a user, user-attribute mappers resolve to placeholder values. Select a user to
            preview real claim data.
          </p>
        </div>

        {optionalScopes.length > 0 && (
          <div className='flex flex-col gap-2'>
            <label className='text-sm font-medium'>Optional scopes</label>
            <div className='flex flex-wrap gap-2'>
              {optionalScopes.map((scope) => {
                const active = selectedOptional.includes(scope.name)
                return (
                  <button
                    key={scope.id}
                    type='button'
                    onClick={() => toggleOptional(scope.name)}
                  >
                    <Badge variant={active ? 'default' : 'secondary'}>{scope.name}</Badge>
                  </button>
                )
              })}
            </div>
          </div>
        )}

        <div>
          <Button onClick={handlePreview} disabled={preview.isPending}>
            {preview.isPending ? 'Preparing…' : 'Preview Token'}
          </Button>
        </div>
      </div>

      {result && (
        <div className='flex flex-col gap-6'>
          {/* Active scopes */}
          <div className='flex flex-col gap-2'>
            <h4 className='text-sm font-semibold'>
              Active scopes ({result.active_scopes.length})
            </h4>
            <div className='flex flex-wrap gap-2'>
              {result.active_scopes.map((scope) => (
                <Badge key={scope.name} variant={scope.type === 'Optional' ? 'secondary' : 'default'}>
                  {scope.name}
                  <span className='ml-1.5 font-normal text-muted-foreground'>{scope.type}</span>
                </Badge>
              ))}
              {result.active_scopes.length === 0 && (
                <span className='text-sm text-muted-foreground'>None</span>
              )}
            </div>
          </div>

          {/* Applied protocol mappers */}
          <div className='flex flex-col gap-2'>
            <h4 className='text-sm font-semibold'>
              Applied protocol mappers ({result.applied_mappers.length})
            </h4>
            <div className='overflow-hidden rounded-md border'>
              <table className='w-full text-sm'>
                <thead className='bg-muted/40 text-left'>
                  <tr>
                    <th className='px-4 py-2 font-medium'>Mapper</th>
                    <th className='px-4 py-2 font-medium'>Type</th>
                    <th className='px-4 py-2 font-medium'>Scope</th>
                  </tr>
                </thead>
                <tbody>
                  {result.applied_mappers.map((mapper, i) => (
                    <tr key={`${mapper.scope}-${mapper.mapper}-${i}`} className='border-t'>
                      <td className='px-4 py-2'>{mapper.mapper}</td>
                      <td className='px-4 py-2 text-muted-foreground'>{mapper.type}</td>
                      <td className='px-4 py-2 text-muted-foreground'>{mapper.scope}</td>
                    </tr>
                  ))}
                  {result.applied_mappers.length === 0 && (
                    <tr>
                      <td className='px-4 py-3 text-muted-foreground' colSpan={3}>
                        No protocol mappers apply for this scope set.
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>

          {/* Generated token claims */}
          <div className='flex flex-col gap-3'>
            <h4 className='text-sm font-semibold'>Generated token claims</h4>
            <JsonPanel title='Access token' value={result.access_token_claims} />
            <JsonPanel title='ID token' value={result.id_token_claims} />
            <JsonPanel title='Userinfo' value={result.userinfo_claims} />
          </div>
        </div>
      )}
    </div>
  )
}
