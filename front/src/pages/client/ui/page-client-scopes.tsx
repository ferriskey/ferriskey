import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useGetClientScopes } from '@/api/client.api'

import { useState } from 'react'
import AddScopeModal from './components/add-scope-modal'
import AssignedScopesTable from './components/assigned-scopes-table'
import EffectiveScopesPreview from './components/effective-scopes-preview'
import { OverviewList } from '@/components/ui/overview-list'
import { Badge } from '@/components/ui/badge'
import { Link } from 'react-router-dom'
import { CLIENT_SCOPES_URL } from '@/routes/sub-router/client-scope.router'
import { Schemas } from '@/api/api.client'

export default function PageClientScopes() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const [isAddModalOpen, setIsAddModalOpen] = useState(false)

  const { data: clientScopesData, isLoading } = useGetClientScopes({
    realm: realm_name,
    clientId: client_id,
  })

  // The API returns an array directly, not wrapped in {data: ...}
  // Handle case where response might be an empty object
  const assignedScopes = Array.isArray(clientScopesData) ? clientScopesData : []

  const getScopeType = (scope: Schemas.ClientScope): 'default' | 'optional' => {
    // This is a simplified approach - in a real implementation, you'd need to check
    // the actual mapping type from the backend
    return scope.default_scope_type === 'DEFAULT' ? 'default' : 'optional'
  }

  return (
    <div className='flex flex-col gap-6'>
      {/* Assigned Scopes Section - Using OverviewList pattern */}
      <OverviewList
        data={assignedScopes}
        isLoading={isLoading}
        searchKeys={['name', 'description', 'protocol']}
        searchPlaceholder='Search scopes...'
        title={(count) => `Assigned Scopes (${count})`}
        renderRow={(scope) => {
          const scopeType = getScopeType(scope)
          return (
            <div className='flex items-center justify-between px-8 py-4 border-b last:border-b-0'>
              <div className='flex items-center gap-4 flex-1 min-w-0'>
                <div className='flex flex-col gap-1 truncate'>
                  <Link
                    to={`${CLIENT_SCOPES_URL(realm_name)}/${scope.id}`}
                    className='font-medium hover:underline truncate'
                  >
                    {scope.name}
                  </Link>
                  <div className='text-sm text-muted-foreground truncate'>{scope.description || 'No description'}</div>
                </div>
              </div>
              <div className='flex items-center gap-4 flex-shrink-0'>
                <Badge variant={scopeType === 'default' ? 'default' : 'secondary'}>
                  {scopeType.charAt(0).toUpperCase() + scopeType.slice(1)}
                </Badge>
                <span className='text-sm text-muted-foreground whitespace-nowrap'>{scope.protocol}</span>
              </div>
              <div className='flex items-center gap-2 flex-shrink-0 ml-6'>
                <AssignedScopesTable
                  realm={realm_name}
                  clientId={client_id}
                  scopes={[scope]}
                  isLoading={false}
                  compact={true}
                />
              </div>
            </div>
          )
        }}
        emptyLabel='No scopes assigned to this client.'
        action={{
          label: 'Add Scope',
          onClick: () => setIsAddModalOpen(true)
        }}
      />

      {/* Effective Scopes Preview */}
      <div className='flex flex-col gap-4'>
        <h3 className='text-lg font-medium'>Effective Scopes Preview</h3>
        <EffectiveScopesPreview
          realm={realm_name}
          clientId={client_id}
          assignedScopes={assignedScopes}
        />
      </div>

      {/* Add Scope Modal */}
      <AddScopeModal
        realm={realm_name}
        clientId={client_id}
        isOpen={isAddModalOpen}
        onClose={() => setIsAddModalOpen(false)}
        assignedScopeIds={assignedScopes.map((scope) => scope.id)}
      />
    </div>
  )
}
