import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import PageContainer from '@/components/ui/page-container'

export default function ProvidersLayout() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()

  return (
    <PageContainer>
      <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b flex items-start justify-between gap-4'>
        <div>
          <h1 className='text-2xl font-bold tracking-tight'>Identity Providers</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Manage external authentication sources and SSO integrations.
          </p>
        </div>
        <Button size='sm' onClick={() => navigate(`/realms/${realm_name}/identity-providers/create`)}>
          <Plus className='h-4 w-4' />
          Add Provider
        </Button>
      </div>
    </PageContainer>
  )
}
