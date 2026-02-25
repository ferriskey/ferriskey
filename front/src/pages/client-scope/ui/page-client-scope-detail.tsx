import { Schemas } from '@/api/api.client'

import ClientScope = Schemas.ClientScope

interface PageClientScopeDetailProps {
  scope: ClientScope
  isLoading?: boolean
}

function ValueRow({
  label,
  value,
}: {
  label: string
  value: string
}) {
  return (
    <div className='flex items-start justify-between py-4 border-t'>
      <div className='w-1/3'>
        <p className='text-sm font-medium'>{label}</p>
      </div>
      <div className='w-1/2'>
        <p className='text-sm text-foreground break-words'>{value}</p>
      </div>
    </div>
  )
}

export default function PageClientScopeDetail({ scope, isLoading }: PageClientScopeDetailProps) {
  if (isLoading) {
    return <div className='text-sm text-muted-foreground'>Loading...</div>
  }

  return (
    <div className='flex flex-col gap-8'>
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Client scope details</p>
          <h2 className='text-base font-semibold'>General Information</h2>
        </div>
        <ValueRow label='Name' value={scope.name} />
        <ValueRow label='Description' value={scope.description || '-'} />
        <ValueRow label='Protocol' value={scope.protocol} />
        <ValueRow label='Type' value={scope.is_default ? 'Default' : 'Optional'} />
        <ValueRow label='Created At' value={new Date(scope.created_at).toLocaleString()} />
        <ValueRow label='Updated At' value={new Date(scope.updated_at).toLocaleString()} />
      </div>

      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Client scope attributes</p>
          <h2 className='text-base font-semibold'>Attributes</h2>
        </div>
        {(scope.attributes?.length ?? 0) === 0 ? (
          <div className='py-4 border-t text-sm text-muted-foreground'>No attributes configured.</div>
        ) : (
          scope.attributes?.map((attribute) => (
            <div key={attribute.id} className='py-4 border-t flex items-start justify-between gap-4'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>{attribute.name}</p>
              </div>
              <div className='w-1/2'>
                <p className='text-sm text-foreground break-words'>{attribute.value || '-'}</p>
              </div>
            </div>
          ))
        )}
      </div>

      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Protocol mapper configuration</p>
          <h2 className='text-base font-semibold'>Protocol Mappers</h2>
        </div>
        {(scope.protocol_mappers?.length ?? 0) === 0 ? (
          <div className='py-4 border-t text-sm text-muted-foreground'>No protocol mappers configured.</div>
        ) : (
          scope.protocol_mappers?.map((mapper) => (
            <div key={mapper.id} className='py-4 border-t flex items-start justify-between gap-4'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>{mapper.name}</p>
                <p className='text-xs text-muted-foreground mt-0.5'>mapper_type: {mapper.mapper_type}</p>
              </div>
              <div className='w-1/2 text-sm text-muted-foreground break-words'>
                id: {mapper.id}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  )
}
