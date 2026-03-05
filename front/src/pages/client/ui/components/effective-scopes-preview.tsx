import { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { useGetClient } from '@/api/client.api'

interface EffectiveScopesPreviewProps {
  realm: string | undefined
  clientId: string | undefined
  assignedScopes: any[]
}

export default function EffectiveScopesPreview({ realm, clientId, assignedScopes }: EffectiveScopesPreviewProps) {
  const { data: clientData } = useGetClient({
    realm: realm,
    clientId: clientId,
  })

  const [effectiveScopes, setEffectiveScopes] = useState<any[]>([])

  useEffect(() => {
    // This is a simplified preview - in a real implementation, you'd need to
    // call a backend endpoint to get the computed effective scopes
    // For now, we'll just show the assigned scopes
    setEffectiveScopes(assignedScopes)
  }, [assignedScopes])

  const clientProtocol = clientData?.data.protocol || 'openid-connect'

  return (
    <Card>
      <CardHeader>
        <CardTitle>Token Scope Preview</CardTitle>
        <div className='text-sm text-muted-foreground'>
          This shows what scopes and claims will be included in tokens issued for this client.
        </div>
      </CardHeader>
      <CardContent>
        <div className='flex flex-col gap-4'>
          {/* Protocol Info */}
          <div className='flex items-center gap-2'>
            <span className='text-sm font-medium'>Protocol:</span>
            <Badge variant='outline'>{clientProtocol}</Badge>
          </div>

          {/* Effective Scopes */}
          <div className='flex flex-col gap-3'>
            <h4 className='text-sm font-medium'>Effective Scopes:</h4>
            {effectiveScopes.length === 0 ? (
              <div className='text-sm text-muted-foreground'>No scopes will be included in tokens.</div>
            ) : (
              <div className='flex flex-wrap gap-2'>
                {effectiveScopes.map((scope) => (
                  <Badge key={scope.id} variant='secondary' className='text-sm'>
                    {scope.name}
                  </Badge>
                ))}
              </div>
            )}
          </div>

          {/* Claims Preview */}
          <div className='flex flex-col gap-3'>
            <h4 className='text-sm font-medium'>Claims Preview:</h4>
            {effectiveScopes.length === 0 ? (
              <div className='text-sm text-muted-foreground'>No claims will be included.</div>
            ) : (
              <div className='grid grid-cols-1 md:grid-cols-2 gap-4'>
                {effectiveScopes.map((scope: any) => (
                  <div key={scope.id} className='flex flex-col gap-2'>
                    <div className='flex items-center gap-2'>
                      <span className='font-medium text-sm'>{scope.name}</span>
                      <Badge variant='outline' className='text-xs'>{scope.protocol}</Badge>
                    </div>
                    <div className='text-sm text-muted-foreground ml-4'>
                      {scope.description || 'No description available'}
                    </div>
                    {/* In a real implementation, you'd show the actual claims from protocol mappers */}
                    <div className='text-xs text-muted-foreground ml-4'>
                      Claims: Standard scope claims (implementation-specific)
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}