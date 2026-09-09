import { useMemo, useState } from 'react'
import {
  useAssignScope,
  useEvaluateClientScopes,
  useGetClientScopes,
  useUnassignScope,
} from '@/api/client.api'
import { useGetClientScopes as useGetRealmClientScopes } from '@/api/client-scope.api'
import { useGetUsers } from '@/api/user.api'
import { NEXT_CLIENT_SCOPES_URL } from '@/next/routes'
import { Schemas } from '@/api/api.client'
import ClientScopesTab, { type AssignableScopeType } from '../ui/client-scopes-tab'
import ClientEvaluatePanel from '../ui/client-evaluate-panel'

import Client = Schemas.Client
import ClientScope = Schemas.ClientScope

export interface ClientScopesTabFeatureProps {
  client: Client
  realm: string
}

export default function ClientScopesTabFeature({ client, realm }: ClientScopesTabFeatureProps) {
  const { data: clientScopesData, isLoading } = useGetClientScopes({
    realm,
    clientId: client.id,
  })
  const { data: realmScopesResponse, isLoading: isLoadingAvailable } = useGetRealmClientScopes({
    realm,
  })
  const { data: usersResponse } = useGetUsers({ realm })

  const assignScope = useAssignScope()
  const unassignScope = useUnassignScope()
  const evaluate = useEvaluateClientScopes()

  const [view, setView] = useState('assigned')
  const [addOpen, setAddOpen] = useState(false)
  const [userId, setUserId] = useState('')
  const [selectedOptional, setSelectedOptional] = useState<string[]>([])

  const assignedScopes = useMemo(() => {
    const raw = clientScopesData as unknown
    return Array.isArray(raw) ? (raw as ClientScope[]) : []
  }, [clientScopesData])

  const availableScopes = useMemo(
    () =>
      (realmScopesResponse?.data ?? []).filter(
        (scope) => !assignedScopes.some((assigned) => assigned.id === scope.id)
      ),
    [realmScopesResponse, assignedScopes]
  )

  const optionalScopes = useMemo(
    () => assignedScopes.filter((s) => s.default_scope_type === 'OPTIONAL'),
    [assignedScopes]
  )
  const defaultScopeNames = useMemo(
    () => assignedScopes.filter((s) => s.default_scope_type === 'DEFAULT').map((s) => s.name),
    [assignedScopes]
  )

  const requestedScope = [...new Set([...defaultScopeNames, ...selectedOptional])].join(' ')

  const handleAdd = (scopeId: string, type: AssignableScopeType) =>
    assignScope.mutate({ realm, clientId: client.id, scopeId, type })

  const handleSetType = async (scope: ClientScope, type: AssignableScopeType) => {
    const current: AssignableScopeType = type === 'default' ? 'optional' : 'default'

    await unassignScope.mutateAsync({
      realm,
      clientId: client.id,
      scopeId: scope.id,
      type: current,
    })
    await assignScope.mutateAsync({ realm, clientId: client.id, scopeId: scope.id, type })
  }

  const handleRemove = (scope: ClientScope) => {
    const type: AssignableScopeType | null =
      scope.default_scope_type === 'DEFAULT'
        ? 'default'
        : scope.default_scope_type === 'OPTIONAL'
          ? 'optional'
          : null

    if (!type) return
    unassignScope.mutate({ realm, clientId: client.id, scopeId: scope.id, type })
  }

  const handleEvaluate = () => {
    if (!userId) return
    evaluate.mutate({ realm, clientId: client.id, userId, scope: requestedScope || undefined })
  }

  return (
    <ClientScopesTab
      view={view}
      onViewChange={setView}
      assignedScopes={assignedScopes}
      availableScopes={availableScopes}
      isLoading={isLoading}
      isLoadingAvailable={isLoadingAvailable}
      isAssigning={assignScope.isPending || unassignScope.isPending}
      addOpen={addOpen}
      scopeHref={(scope) => `${NEXT_CLIENT_SCOPES_URL(realm)}/${scope.id}`}
      onAddOpenChange={setAddOpen}
      onAdd={handleAdd}
      onSetType={(scope, type) => void handleSetType(scope, type)}
      onRemove={handleRemove}
      evaluatePanel={
        <ClientEvaluatePanel
          users={usersResponse?.data ?? []}
          optionalScopes={optionalScopes}
          userId={userId}
          selectedOptional={selectedOptional}
          requestedScope={requestedScope}
          isPending={evaluate.isPending}
          result={evaluate.data}
          onUserChange={setUserId}
          onToggleOptional={(name) =>
            setSelectedOptional((prev) =>
              prev.includes(name) ? prev.filter((n) => n !== name) : [...prev, name]
            )
          }
          onEvaluate={handleEvaluate}
        />
      }
    />
  )
}
