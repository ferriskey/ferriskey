import { useState, useMemo } from 'react'
import { ExternalLink, Copy, Check, Info } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import type { ProviderTemplate } from '@/constants/identity-provider-templates'
import ProviderIcon from './provider-icon'

interface ProviderHelpPanelProps {
  template: ProviderTemplate
  callbackUrl: string
}

export default function ProviderHelpPanel({
  template,
  callbackUrl,
}: ProviderHelpPanelProps) {
  const [copied, setCopied] = useState(false)

  const handleCopyCallback = async () => {
    await navigator.clipboard.writeText(callbackUrl)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const setupSteps = useMemo(() => {
    const isCustom = template.id === 'custom'

    if (isCustom) {
      return [
        'Access your OAuth provider\'s developer console',
        'Create a new OAuth application',
        'Copy the Client ID and Client Secret',
        'Add the Redirect URI to your OAuth app',
        'Configure authorization and token URLs',
      ]
    }

    return [
      `Go to ${template.displayName} developer console`,
      'Create a new OAuth application',
      'Add the Redirect URI below',
      'Copy your Client ID and Client Secret',
      'Paste them in the form',
    ]
  }, [template])

  return (
    <div className='space-y-4'>
      {/* Provider Info Card */}
      <Card className='sticky top-24'>
        <CardHeader className='pb-3'>
          <div className='flex items-center gap-3'>
            <div className='h-10 w-10 flex items-center justify-center rounded-lg bg-primary/10'>
              <ProviderIcon icon={template.icon} size='md' />
            </div>
            <div>
              <CardTitle className='text-base'>{template.displayName}</CardTitle>
              <p className='text-xs text-muted-foreground mt-0.5'>
                {template.provider_type.toUpperCase()} Provider
              </p>
            </div>
          </div>
        </CardHeader>

        <CardContent className='space-y-4 pt-2'>
          {/* Documentation Link */}
          {template.documentation_url && (
            <div>
              <Button
                variant='outline'
                size='sm'
                className='w-full justify-between'
                asChild
              >
                <a
                  href={template.documentation_url}
                  target='_blank'
                  rel='noopener noreferrer'
                >
                  <span>View Setup Guide</span>
                  <ExternalLink className='h-3 w-3' />
                </a>
              </Button>
            </div>
          )}

          {/* Redirect URI */}
          <div className='space-y-2'>
            <div className='flex items-center justify-between'>
              <h4 className='text-xs font-semibold text-muted-foreground uppercase tracking-wide'>
                Redirect URI
              </h4>
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant='ghost'
                      size='sm'
                      onClick={handleCopyCallback}
                      className='h-6 w-6 p-0'
                    >
                      {copied ? (
                        <Check className='h-3 w-3 text-green-500' />
                      ) : (
                        <Copy className='h-3 w-3' />
                      )}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>{copied ? 'Copied!' : 'Copy to clipboard'}</p>
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            </div>
            <div className='p-2 bg-muted rounded-md'>
              <code className='text-[10px] break-all leading-relaxed'>
                {callbackUrl}
              </code>
            </div>
            <p className='text-[10px] text-muted-foreground leading-relaxed'>
              Add this URL to your {template.displayName} OAuth app's allowed redirect URIs.
            </p>
          </div>

          {/* Setup Checklist */}
          <div className='space-y-2'>
            <h4 className='text-xs font-semibold text-muted-foreground uppercase tracking-wide'>
              Setup Steps
            </h4>
            <ol className='space-y-1.5'>
              {setupSteps.map((step, index) => (
                <li key={index} className='flex gap-2 text-xs'>
                  <span className='text-muted-foreground shrink-0 font-medium'>
                    {index + 1}.
                  </span>
                  <span className='text-muted-foreground leading-relaxed'>
                    {step}
                  </span>
                </li>
              ))}
            </ol>
          </div>

          {/* Tips */}
          <Alert>
            <Info className='h-3.5 w-3.5' />
            <AlertDescription className='text-xs'>
              {template.id === 'custom' ? (
                <>
                  Make sure your OAuth provider supports the authorization code flow.
                  The userinfo URL is optional if user data is included in the token response.
                </>
              ) : (
                <>
                  The OAuth URLs are pre-configured for {template.displayName}. You only
                  need to provide your credentials from their developer console.
                </>
              )}
            </AlertDescription>
          </Alert>

          {/* Default Scopes */}
          {template.default_scopes.length > 0 && (
            <div className='space-y-2'>
              <h4 className='text-xs font-semibold text-muted-foreground uppercase tracking-wide'>
                Default Scopes
              </h4>
              <div className='p-2 bg-muted rounded-md'>
                <code className='text-[10px] break-all leading-relaxed'>
                  {template.default_scopes.join(' ')}
                </code>
              </div>
              <p className='text-[10px] text-muted-foreground leading-relaxed'>
                These scopes will be requested during authentication.
              </p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
