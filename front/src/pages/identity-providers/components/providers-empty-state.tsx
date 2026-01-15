import { ScanFace, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { PROVIDER_TEMPLATES } from '@/constants/identity-provider-templates'
import ProviderIcon from './provider-icon'

interface ProvidersEmptyStateProps {
  onCreateProvider: () => void
}

// Show top 4 popular providers
const popularProviders = PROVIDER_TEMPLATES.filter((t) =>
  ['google', 'discord', 'github', 'microsoft'].includes(t.id)
)

export default function ProvidersEmptyState({ onCreateProvider }: ProvidersEmptyStateProps) {
  return (
    <Card className='border-dashed'>
      <CardContent className='flex flex-col items-center justify-center py-12 px-8 text-center'>
        <div className='h-14 w-14 rounded-full bg-muted flex items-center justify-center mb-4'>
          <ScanFace className='h-7 w-7 text-muted-foreground' />
        </div>

        <h3 className='text-lg font-semibold mb-2'>No Identity Providers Yet</h3>
        <p className='text-muted-foreground max-w-md mb-6'>
          Connect external authentication providers to allow users to sign in with their existing accounts from Google, Discord, GitHub, and more.
        </p>

        <Button size='lg' onClick={onCreateProvider} className='mb-6'>
          <Plus className='h-4 w-4 mr-2' />
          Add Your First Provider
        </Button>

        <div className='space-y-3'>
          <p className='text-sm text-muted-foreground'>Popular providers:</p>
          <div className='flex items-center justify-center gap-3'>
            {popularProviders.map((provider) => (
              <button
                key={provider.id}
                onClick={onCreateProvider}
                className='flex flex-col items-center gap-1.5 p-2.5 rounded-lg hover:bg-muted transition-colors'
              >
                <div className='h-9 w-9 flex items-center justify-center'>
                  <ProviderIcon icon={provider.icon} size='md' />
                </div>
                <span className='text-xs text-muted-foreground'>{provider.displayName}</span>
              </button>
            ))}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
