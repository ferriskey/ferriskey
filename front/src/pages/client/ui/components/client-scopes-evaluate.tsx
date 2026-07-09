import { useMemo, useState } from 'react'
import { Schemas } from '@/api/api.client'
import { useEvaluateClientScopes } from '@/api/client.api'
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

  const [userId, setUserId] = useState<string>('')
  const [selectedOptional, setSelectedOptional] = useState<string[]>([])

  const evaluate = useEvaluateClientScopes()

  const toggleOptional = (name: string) => {
    setSelectedOptional((prev) =>
      prev.includes(name) ? prev.filter((n) => n !== name) : [...prev, name]
    )
  }

  const handleEvaluate = () => {
    if (!realm || !clientId || !userId) return
    // Default scopes always apply on the backend; include them in the requested scope string
    // so the `scope` claim is realistic, plus the optional scopes the admin selected.
    const scope = [...new Set([...defaultScopeNames, ...selectedOptional])].join(' ')
    evaluate.mutate({ realm, clientId, userId, scope: scope || undefined })
  }

  const result = evaluate.data

  return (
    <div className='flex flex-col gap-6'>
      <div className='flex flex-col gap-4 rounded-md border p-4'>
        <div className='flex flex-col gap-2'>
          <label className='text-sm font-medium'>User</label>
          <Select value={userId} onValueChange={setUserId}>
            <SelectTrigger className='max-w-md'>
              <SelectValue placeholder='Select a user to evaluate' />
            </SelectTrigger>
            <SelectContent>
              {users.map((user) => (
                <SelectItem key={user.id} value={user.id}>
                  {user.username}
                  {user.email ? ` (${user.email})` : ''}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
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
          <Button onClick={handleEvaluate} disabled={!userId || evaluate.isPending}>
            {evaluate.isPending ? 'Evaluating…' : 'Evaluate'}
          </Button>
        </div>
      </div>

      {result && (
        <div className='flex flex-col gap-6'>
          {/* Effective protocol mappers */}
          <div className='flex flex-col gap-2'>
            <h4 className='text-sm font-semibold'>
              Effective protocol mappers ({result.effective_mappers.length})
            </h4>
            <div className='overflow-hidden rounded-md border'>
              <table className='w-full text-sm'>
                <thead className='bg-muted/40 text-left'>
                  <tr>
                    <th className='px-4 py-2 font-medium'>Name</th>
                    <th className='px-4 py-2 font-medium'>Type</th>
                  </tr>
                </thead>
                <tbody>
                  {result.effective_mappers.map((mapper, i) => (
                    <tr key={`${mapper.name}-${i}`} className='border-t'>
                      <td className='px-4 py-2'>{mapper.name}</td>
                      <td className='px-4 py-2 text-muted-foreground'>{mapper.mapper_type}</td>
                    </tr>
                  ))}
                  {result.effective_mappers.length === 0 && (
                    <tr>
                      <td className='px-4 py-3 text-muted-foreground' colSpan={2}>
                        No protocol mappers apply for this scope set.
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>

          {/* Effective roles */}
          <div className='flex flex-col gap-2'>
            <h4 className='text-sm font-semibold'>Effective roles</h4>
            <div className='flex flex-col gap-3 rounded-md border p-4'>
              <div className='flex flex-col gap-1'>
                <span className='text-xs text-muted-foreground'>Realm roles</span>
                <div className='flex flex-wrap gap-2'>
                  {result.effective_roles.realm_roles.length > 0 ? (
                    result.effective_roles.realm_roles.map((role) => (
                      <Badge key={role} variant='secondary'>
                        {role}
                      </Badge>
                    ))
                  ) : (
                    <span className='text-sm text-muted-foreground'>None</span>
                  )}
                </div>
              </div>
              {Object.entries(result.effective_roles.client_roles).map(([client, roles]) => (
                <div key={client} className='flex flex-col gap-1'>
                  <span className='text-xs text-muted-foreground'>{client}</span>
                  <div className='flex flex-wrap gap-2'>
                    {roles.map((role) => (
                      <Badge key={`${client}-${role}`} variant='secondary'>
                        {role}
                      </Badge>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Generated tokens */}
          <div className='flex flex-col gap-3'>
            <h4 className='text-sm font-semibold'>Generated token claims</h4>
            <JsonPanel title='Access token' value={result.access_token} />
            <JsonPanel title='ID token' value={result.id_token} />
            <JsonPanel title='Userinfo' value={result.userinfo} />
          </div>
        </div>
      )}
    </div>
  )
}
