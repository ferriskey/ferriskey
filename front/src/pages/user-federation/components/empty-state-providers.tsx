import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Database, Server, Key } from 'lucide-react'

interface EmptyStateProvidersProps {
  onCreateProvider: (type: 'LDAP' | 'Kerberos') => void
}

export default function EmptyStateProviders({ onCreateProvider }: EmptyStateProvidersProps) {
  const providerTypes = [
    {
      type: 'LDAP' as const,
      title: 'LDAP',
      description: 'Connect to an LDAP directory for user authentication and federation',
      icon: Server,
      features: ['User synchronization', 'Group mapping', 'Attribute mapping'],
      disabled: false,
    },
    {
      type: 'Kerberos' as const,
      title: 'Kerberos',
      description: 'Integrate with Kerberos for secure network authentication',
      icon: Key,
      features: ['SSO support', 'Ticket validation', 'Cross-realm authentication'],
      disabled: true,
    },
  ]

  return (
    <div className='flex flex-col items-center justify-center py-12 px-4'>
      <div className='flex h-16 w-16 items-center justify-center rounded-full bg-primary/10 mb-6'>
        <Database className='h-8 w-8 text-primary' />
      </div>

      <h2 className='text-2xl font-semibold mb-2'>No Federation Providers</h2>
      <p className='text-muted-foreground text-center mb-8 max-w-md'>
        Get started by creating your first federation provider to connect external user directories
      </p>

      <div className='grid grid-cols-1 md:grid-cols-2 gap-6 w-full max-w-4xl'>
        {providerTypes.map((provider) => {
          const Icon = provider.icon
          return (
            <Card
              key={provider.type}
              className={`transition-all ${provider.disabled
                ? 'opacity-60 cursor-not-allowed'
                : 'hover:shadow-lg cursor-pointer group'
                }`}
              onClick={() => !provider.disabled && onCreateProvider(provider.type)}
            >
              <CardContent className='p-6'>
                <div className='flex items-start gap-4'>
                  <div className={`flex h-12 w-12 items-center justify-center rounded-lg shrink-0 transition-colors ${provider.disabled
                    ? 'bg-muted'
                    : 'bg-primary/10 group-hover:bg-primary/20'
                    }`}>
                    <Icon className={`h-6 w-6 ${provider.disabled ? 'text-muted-foreground' : 'text-primary'
                      }`} />
                  </div>

                  <div className='flex-1'>
                    <div className='flex items-center gap-2 mb-2'>
                      <h3 className='font-semibold text-lg'>{provider.title}</h3>
                      {provider.disabled && (
                        <Badge variant='secondary' className='text-xs'>
                          Coming Soon
                        </Badge>
                      )}
                    </div>
                    <p className='text-sm text-muted-foreground mb-4'>
                      {provider.description}
                    </p>

                    <ul className='space-y-1 mb-4'>
                      {provider.features.map((feature, idx) => (
                        <li key={idx} className='text-sm text-muted-foreground flex items-center gap-2'>
                          <span className='h-1 w-1 rounded-full bg-primary' />
                          {feature}
                        </li>
                      ))}
                    </ul>

                    <Button
                      variant='outline'
                      size='sm'
                      className={`w-full ${!provider.disabled && 'group-hover:bg-secondary'
                        } transition-colors`}
                      disabled={provider.disabled}
                    >
                      Create {provider.title} Provider
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          )
        })}
      </div>
    </div>
  )
}
