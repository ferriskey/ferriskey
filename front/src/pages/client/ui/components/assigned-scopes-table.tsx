import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Button } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { ChevronDown } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { useAssignScope, useUnassignScope } from '@/api/client.api'
import { Link } from 'react-router-dom'
import {
  CLIENT_SCOPE_DETAILS_URL,
  CLIENT_SCOPE_URL,
} from '@/routes/sub-router/client-scope.router'
import { Schemas } from '@/api/api.client'

interface AssignedScopesTableProps {
  realm: string | undefined
  clientId: string | undefined
  scopes: Schemas.ClientScope[]
  isLoading: boolean
  compact?: boolean
}

export default function AssignedScopesTable({ realm, clientId, scopes, isLoading, compact = false }: AssignedScopesTableProps) {
  const assignScope = useAssignScope()
  const unassignScope = useUnassignScope()

  const handleChangeType = async (scopeId: string, currentType: 'default' | 'optional' | 'none') => {
    if (currentType === 'default' || currentType === 'optional') {
      const newType = currentType === 'default' ? 'optional' : 'default'
      await unassignScope.mutateAsync({
        realm: realm!,
        clientId: clientId!,
        scopeId: scopeId,
        type: currentType
      })

      // Only assign the new type if unassign succeeded
      await assignScope.mutateAsync({
        realm: realm!,
        clientId: clientId!,
        scopeId: scopeId,
        type: newType
      })
    }
  }

  const handleRemove = (scopeId: string, scopeType: 'default' | 'optional') => {
    unassignScope.mutate({
      realm: realm!,
      clientId: clientId!,
      scopeId: scopeId,
      type: scopeType
    })
  }

  const getScopeType = (scope: Schemas.ClientScope): 'default' | 'optional' => {
    // This is a simplified approach - in a real implementation, you'd need to check
    // the actual mapping type from the backend
    if (scope.default_scope_type === 'DEFAULT') {
      return 'default'
    }
    // For now, assume it's optional if not default
    return 'optional'
  }

  if (compact) {
    // Compact mode for use within OverviewList rows
    if (scopes.length !== 1) return null
    const scope = scopes[0]
    const scopeType = getScopeType(scope)

    return (
      <div className='flex items-center gap-2'>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant='outline' size='sm' className='whitespace-nowrap'>
              {scopeType.charAt(0).toUpperCase() + scopeType.slice(1)}
              <ChevronDown className='ml-2 h-4 w-4' />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem onClick={() => handleChangeType(scope.id, scopeType)}>
              Change to {scopeType === 'default' ? 'Optional' : 'Default'}
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => handleRemove(scope.id, scopeType)}>
              Remove
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    )
  }

  if (isLoading) {
    return <div>Loading scopes...</div>
  }

  if (scopes.length === 0) {
    return <div className='text-muted-foreground'>No scopes assigned to this client.</div>
  }

  return (
    <div className='border rounded-lg'>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Description</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Protocol</TableHead>
            <TableHead>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {scopes.map((scope) => {
            const scopeType = getScopeType(scope)
            return (
              <TableRow key={scope.id}>
                <TableCell>
                  <Link
                    to={`${CLIENT_SCOPE_URL(realm, scope.id)}${CLIENT_SCOPE_DETAILS_URL}`}
                    className='hover:underline'
                  >
                    {scope.name}
                  </Link>
                </TableCell>
                <TableCell>{scope.description || '-'}</TableCell>
                <TableCell>
                  <Badge variant={scopeType === 'default' ? 'default' : scopeType === 'optional' ? 'secondary' : 'outline'}>
                    {scopeType.charAt(0).toUpperCase() + scopeType.slice(1)}
                  </Badge>
                </TableCell>
                <TableCell>{scope.protocol}</TableCell>
                <TableCell>
                  <div className='flex items-center gap-2'>
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant='outline' size='sm'>
                          {scopeType.charAt(0).toUpperCase() + scopeType.slice(1)}
                          <ChevronDown className='ml-2 h-4 w-4' />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent>
                        <DropdownMenuItem onClick={() => handleChangeType(scope.id, scopeType)}>
                          Change to {scopeType === 'default' ? 'Optional' : 'Default'}
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => handleRemove(scope.id, scopeType)}>
                          Remove
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </TableCell>
              </TableRow>
            )
          })}
        </TableBody>
      </Table>
    </div>
  )
}
