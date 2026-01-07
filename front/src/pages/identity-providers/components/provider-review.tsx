import { Check, ExternalLink } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import type { ProviderTemplate } from '@/constants/identity-provider-templates'
import type { ProviderFormData } from './provider-config-form'
import ProviderIcon from './provider-icon'

interface ProviderReviewProps {
  template: ProviderTemplate
  formData: ProviderFormData
  callbackUrl: string
}

export default function ProviderReview({
  template,
  formData,
  callbackUrl,
}: ProviderReviewProps) {
  const configItems = [
    {
      label: 'Authorization URL',
      value: formData.authorizationUrl || template.authorization_url,
    },
    {
      label: 'Token URL',
      value: formData.tokenUrl || template.token_url,
    },
    {
      label: 'User Info URL',
      value: formData.userinfoUrl || template.userinfo_url || 'Not configured',
    },
    {
      label: 'Scopes',
      value: formData.scopes || template.default_scopes.join(' ') || 'Default',
    },
  ]

  return (
    <div className='space-y-4'>
      {/* Provider Summary */}
      <Card>
        <CardHeader className='pb-3'>
          <div className='flex items-center gap-4'>
            <div className='h-12 w-12 flex items-center justify-center rounded-lg bg-primary/10'>
              <ProviderIcon icon={template.icon} size='md' />
            </div>
            <div className='flex-1'>
              <CardTitle className='text-lg'>{formData.displayName || template.displayName}</CardTitle>
              <div className='flex items-center gap-2 mt-1'>
                <Badge variant='outline'>{template.provider_type.toUpperCase()}</Badge>
                <Badge variant='secondary' className='bg-green-100 text-green-700'>
                  <Check className='h-3 w-3 mr-1' />
                  Ready to create
                </Badge>
              </div>
            </div>
          </div>
        </CardHeader>
      </Card>

      {/* Configuration Summary */}
      <Card>
        <CardHeader className='pb-3'>
          <CardTitle className='text-base'>Configuration Summary</CardTitle>
        </CardHeader>
        <CardContent className='space-y-3'>
          {/* Credentials */}
          <div className='space-y-3'>
            <h4 className='text-sm font-medium text-muted-foreground'>Credentials</h4>
            <div className='grid gap-2'>
              <ReviewItem label='Client ID' value={formData.clientId} masked={false} />
              <ReviewItem label='Client Secret' value={formData.clientSecret} masked={true} />
            </div>
          </div>

          <Separator />

          {/* OAuth Configuration */}
          <div className='space-y-3'>
            <h4 className='text-sm font-medium text-muted-foreground'>OAuth Configuration</h4>
            <div className='grid gap-2'>
              {configItems.map((item) => (
                <ReviewItem
                  key={item.label}
                  label={item.label}
                  value={item.value}
                  masked={false}
                />
              ))}
            </div>
          </div>

          <Separator />

          {/* Callback URL */}
          <div className='space-y-3'>
            <h4 className='text-sm font-medium text-muted-foreground'>Redirect URI</h4>
            <div className='p-3 bg-muted rounded-md'>
              <code className='text-xs break-all'>{callbackUrl}</code>
            </div>
            <p className='text-xs text-muted-foreground'>
              Make sure this URL is configured in your {template.displayName} OAuth app settings.
            </p>
          </div>
        </CardContent>
      </Card>

      {/* Documentation Link */}
      {template.documentation_url && (
        <Card className='bg-muted/50'>
          <CardContent className='p-4'>
            <div className='flex items-center justify-between'>
              <div className='space-y-1'>
                <p className='text-sm font-medium'>Need help with {template.displayName}?</p>
                <p className='text-xs text-muted-foreground'>
                  Check out the official documentation for setup instructions.
                </p>
              </div>
              <Button variant='outline' size='sm' asChild>
                <a
                  href={template.documentation_url}
                  target='_blank'
                  rel='noopener noreferrer'
                  className='flex items-center gap-2'
                >
                  <span>View Docs</span>
                  <ExternalLink className='h-3 w-3' />
                </a>
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}

interface ReviewItemProps {
  label: string
  value: string
  masked: boolean
}

function ReviewItem({ label, value, masked }: ReviewItemProps) {
  const displayValue = masked && value ? '••••••••••••••••' : value || '—'

  return (
    <div className='flex items-center justify-between py-1'>
      <span className='text-sm text-muted-foreground'>{label}</span>
      <span className='text-sm font-mono truncate max-w-[60%] text-right'>
        {displayValue}
      </span>
    </div>
  )
}
