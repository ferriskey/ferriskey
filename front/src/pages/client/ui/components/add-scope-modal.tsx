import { useState } from 'react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useGetClientScopes as useGetRealmClientScopes } from '@/api/client-scope.api'
import { useAssignScope } from '@/api/client.api'
import { Checkbox } from '@/components/ui/checkbox'
import { Schemas } from '@/api/api.client'

interface AddScopeModalProps {
  realm: string | undefined
  clientId: string | undefined
  isOpen: boolean
  onClose: () => void
  assignedScopeIds: string[]
}

export default function AddScopeModal({ realm, clientId, isOpen, onClose, assignedScopeIds }: AddScopeModalProps) {
  const [selectedScopeId, setSelectedScopeId] = useState<string | null>(null)
  const [scopeType, setScopeType] = useState<'default' | 'optional'>('default')
  const [searchTerm, setSearchTerm] = useState('')

  const { data: realmScopesData, isLoading: isLoadingRealmScopes } = useGetRealmClientScopes({
    realm: realm!,
  })

  const assignScope = useAssignScope()

  const availableScopes = realmScopesData?.data?.filter(
    (scope: Schemas.ClientScope) => !assignedScopeIds.includes(scope.id)
  ) || []

  const filteredScopes = availableScopes.filter(
    (scope: Schemas.ClientScope) =>
      scope.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      scope.description?.toLowerCase().includes(searchTerm.toLowerCase())
  )

  const handleAssign = () => {
    if (!selectedScopeId) return

    assignScope.mutate({
      realm: realm!,
      clientId: clientId!,
      scopeId: selectedScopeId,
      type: scopeType
    }, {
      onSuccess: () => {
        onClose()
        setSelectedScopeId(null)
        setScopeType('default')
        setSearchTerm('')
      },
    })
  }

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className='max-w-2xl max-h-[80vh] overflow-y-auto'>
        <DialogHeader>
          <DialogTitle>Add Client Scope</DialogTitle>
        </DialogHeader>

        <div className='flex flex-col gap-4 py-4'>
          {/* Search */}
          <div className='flex gap-2'>
            <Input
              placeholder='Search scopes...'
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className='flex-1'
            />
            <Select value={scopeType} onValueChange={(value: 'default' | 'optional') => setScopeType(value)}>
              <SelectTrigger className='w-[180px]'>
                <SelectValue placeholder='Scope Type' />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value='default'>Default</SelectItem>
                <SelectItem value='optional'>Optional</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Scope List */}
          <div className='flex flex-col gap-2'>
            {isLoadingRealmScopes ? (
              <div className='text-muted-foreground'>Loading available scopes...</div>
            ) : filteredScopes.length === 0 ? (
              <div className='text-muted-foreground'>No available scopes to assign.</div>
            ) : (
              <div className='space-y-2'>
                {filteredScopes.map((scope: Schemas.ClientScope) => (
                  <div
                    key={scope.id}
                    className={`flex items-start gap-3 p-3 border rounded-lg cursor-pointer hover:bg-muted/50 ${
                      selectedScopeId === scope.id ? 'border-primary bg-muted/50' : 'border-border'
                    }`}
                    onClick={() => setSelectedScopeId(scope.id)}
                  >
                    <Checkbox
                      checked={selectedScopeId === scope.id}
                      onCheckedChange={() => setSelectedScopeId(scope.id)}
                      className='mt-1'
                    />
                    <div className='flex-1'>
                      <div className='font-medium'>{scope.name}</div>
                      <div className='text-sm text-muted-foreground'>{scope.description || 'No description'}</div>
                      <div className='text-xs text-muted-foreground mt-1'>
                        Protocol: {scope.protocol}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant='outline' onClick={onClose} disabled={assignScope.isPending}>
            Cancel
          </Button>
          <Button
            onClick={handleAssign}
            disabled={!selectedScopeId || assignScope.isPending}
          >
            {assignScope.isPending ? 'Assigning...' : 'Assign Scope'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
