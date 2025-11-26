import { ReactNode } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { ExternalLink, Github } from 'lucide-react'
import { cn } from '@/lib/utils'

interface BulletPoint {
  text: string
  highlight?: string
}

interface ComingSoonProps {
  children: ReactNode
  title?: string
  description?: string
  badgeText?: string
  bulletPoints?: BulletPoint[]
  imageSrc?: string
  imageAlt?: string
  blurIntensity?: 'light' | 'medium' | 'heavy'
  className?: string
}

export function ComingSoon({
  children,
  title = 'Observability is here!',
  description = 'We\'ve just released our brand-new Observability feature, now available for everyone.',
  badgeText = 'Coming Soon',
  bulletPoints = [
    { text: '1-click setup:', highlight: 'Dev + Ops friendly' },
    { text: 'Zero maintenance:', highlight: 'Qovery manages it for you within your infrastructure' },
    { text: 'Your data stays in your infrastructure' },
    { text: 'Correlated infrastructure and application events:', highlight: 'troubleshoot with ease' },
    { text: 'Faster recovery:', highlight: 'strong MTTR (Mean Time to Repair) reduction' },
    { text: 'Built to be used by Software Engineers:', highlight: 'troubleshoot and recover directly from your application' }
  ],
  imageSrc = '/api/placeholder/400/300',
  imageAlt = 'Feature preview',
  blurIntensity = 'medium',
  className
}: ComingSoonProps) {
  const blurClasses = {
    light: 'backdrop-blur-sm',
    medium: 'backdrop-blur-md',
    heavy: 'backdrop-blur-lg'
  }

  return (
    <div className={cn('relative min-h-screen', className)}>
      {/* Background content with gradient fade */}
      <div className={cn(
        'relative transition-all duration-700',
        blurClasses[blurIntensity],
        'pointer-events-none select-none'
      )}>
        {/* Gradient overlay for fade effect */}
        <div className='absolute inset-0 bg-gradient-to-b from-background/40 via-background/60 to-background/90 z-10' />
        <div className='relative z-0 opacity-30'>
          {children}
        </div>
      </div>

      {/* Main overlay */}
      <div className='absolute inset-0 flex items-center justify-center p-4 z-20'>
        <div className='w-full max-w-5xl'>
          <Card className='shadow-2xl border-2 bg-card/95 backdrop-blur-sm overflow-hidden'>
            <div className='flex flex-col lg:flex-row'>
              {/* Content Section */}
              <div className='flex-1 lg:flex-[2]'>
                <CardHeader className='space-y-4 pb-6'>
                  <div className='flex items-center gap-2'>
                    <Badge variant='secondary' className='bg-primary/10 text-primary border-primary/20'>
                      {badgeText}
                    </Badge>
                  </div>
                  <CardTitle className='text-3xl font-bold tracking-tight'>
                    {title}
                  </CardTitle>
                  <p className='text-muted-foreground text-lg leading-relaxed'>
                    {description}
                  </p>
                </CardHeader>

                <CardContent className='space-y-6 pt-0'>
                  {/* Custom bullet points */}
                  {bulletPoints && bulletPoints.length > 0 && (
                    <div className='space-y-3'>
                      {bulletPoints.map((point, index) => (
                        <div key={index} className='flex items-start gap-3'>
                          <div className='size-2 rounded-full bg-primary mt-2.5 flex-shrink-0' />
                          <span className='text-sm leading-relaxed'>
                            {point.highlight ? (
                              <>
                                <strong>{point.text}</strong> {point.highlight}
                              </>
                            ) : (
                              <strong>{point.text}</strong>
                            )}
                          </span>
                        </div>
                      ))}
                    </div>
                  )}

                  {/* Action buttons */}
                  <div className='flex flex-col sm:flex-row gap-3 pt-6'>
                    <Button
                      size='lg'
                      className='bg-primary hover:bg-primary/90'
                      asChild
                    >
                      <a
                        href='https://ferriskey.rs'
                        target='_blank'
                        rel='noopener noreferrer'
                        className='flex items-center'
                      >
                        <ExternalLink className='size-4 mr-2' />
                        Visit ferriskey.rs
                      </a>
                    </Button>
                    <Button variant='outline' size='lg' asChild>
                      <a
                        href='https://github.com/ferriskey/ferriskey'
                        target='_blank'
                        rel='noopener noreferrer'
                        className='flex items-center'
                      >
                        <Github className='size-4 mr-2' />
                        View on GitHub
                      </a>
                    </Button>
                  </div>
                </CardContent>
              </div>

              {/* Image Section */}
              <div className='flex-1 lg:border-l'>
                <div className='relative h-64 lg:h-full min-h-[400px]'>
                  <img
                    src={imageSrc}
                    alt={imageAlt}
                    className='absolute inset-0 w-full h-full object-cover'
                  />
                  <div className='absolute inset-0 bg-gradient-to-br from-primary/20 to-purple-600/20' />

                  {/* Optional overlay content on image */}
                  <div className='absolute inset-0 flex flex-col justify-end p-6 text-white'>
                    <div className='bg-black/20 backdrop-blur-sm rounded-lg p-4'>
                      <h3 className='font-semibold text-lg mb-2'>Preview Available Soon</h3>
                      <p className='text-sm text-white/90'>
                        Get a sneak peek at what's coming to FerrisKey
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </Card>
        </div>
      </div>
    </div>
  )
}
