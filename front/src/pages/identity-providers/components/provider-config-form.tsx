import { useState } from 'react'
import { UseFormReturn } from 'react-hook-form'
import { ChevronDown, ChevronUp, Eye, EyeOff } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import {
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible'
import type { ProviderTemplate } from '@/constants/identity-provider-templates'

export interface ProviderFormData {
  displayName: string
  clientId: string
  clientSecret: string
  authorizationUrl?: string
  tokenUrl?: string
  userinfoUrl?: string
  scopes?: string
}

interface ProviderConfigFormProps {
  template: ProviderTemplate
  form: UseFormReturn<ProviderFormData>
  callbackUrl: string
}

export default function ProviderConfigForm({
  template,
  form,
}: ProviderConfigFormProps) {
  const [showSecret, setShowSecret] = useState(false)
  const [showAdvanced, setShowAdvanced] = useState(template.id === 'custom')

  const isCustomProvider = template.id === 'custom'


  return (
    <div className='space-y-4'>
      {/* Provider Header - Removed since it's now in the help panel */}

      {/* Basic Configuration */}
      <div className='space-y-3'>
        <h4 className='font-medium'>Basic Configuration</h4>

        <FormField
          control={form.control}
          name='displayName'
          render={({ field }) => (
            <FormItem>
              <FormLabel>Display Name</FormLabel>
              <FormControl>
                <Input placeholder={template.displayName} {...field} />
              </FormControl>
              <FormDescription>
                This will be shown to users on the login page
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name='clientId'
          render={({ field }) => (
            <FormItem>
              <FormLabel>Client ID *</FormLabel>
              <FormControl>
                <Input
                  placeholder={`Enter your ${template.displayName} Client ID`}
                  {...field}
                />
              </FormControl>
              <FormDescription>
                The OAuth Client ID from your {template.displayName} app
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name='clientSecret'
          render={({ field }) => (
            <FormItem>
              <FormLabel>Client Secret *</FormLabel>
              <FormControl>
                <div className='relative'>
                  <Input
                    type={showSecret ? 'text' : 'password'}
                    placeholder={`Enter your ${template.displayName} Client Secret`}
                    className='pr-10'
                    {...field}
                  />
                  <Button
                    type='button'
                    variant='ghost'
                    size='sm'
                    className='absolute right-0 top-0 h-full px-3 hover:bg-transparent'
                    onClick={() => setShowSecret(!showSecret)}
                  >
                    {showSecret ? (
                      <EyeOff className='h-4 w-4 text-muted-foreground' />
                    ) : (
                      <Eye className='h-4 w-4 text-muted-foreground' />
                    )}
                  </Button>
                </div>
              </FormControl>
              <FormDescription>
                The OAuth Client Secret from your {template.displayName} app
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
      </div>

      {/* Advanced Settings */}
      <Collapsible open={showAdvanced} onOpenChange={setShowAdvanced}>
        <CollapsibleTrigger asChild>
          <Button
            variant='ghost'
            className='flex items-center gap-2 w-full justify-between p-0 h-auto hover:bg-transparent'
          >
            <span className='text-sm font-medium'>
              Advanced Settings {!isCustomProvider && '(Optional)'}
            </span>
            {showAdvanced ? (
              <ChevronUp className='h-4 w-4' />
            ) : (
              <ChevronDown className='h-4 w-4' />
            )}
          </Button>
        </CollapsibleTrigger>

        <CollapsibleContent className='space-y-3 pt-3'>
          {!isCustomProvider && (
            <p className='text-sm text-muted-foreground'>
              These values are pre-configured for {template.displayName}. Only modify if you know what you're doing.
            </p>
          )}

          <FormField
            control={form.control}
            name='authorizationUrl'
            render={({ field }) => (
              <FormItem>
                <FormLabel>Authorization URL {isCustomProvider && '*'}</FormLabel>
                <FormControl>
                  <Input
                    placeholder='https://provider.com/oauth/authorize'
                    {...field}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name='tokenUrl'
            render={({ field }) => (
              <FormItem>
                <FormLabel>Token URL {isCustomProvider && '*'}</FormLabel>
                <FormControl>
                  <Input
                    placeholder='https://provider.com/oauth/token'
                    {...field}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name='userinfoUrl'
            render={({ field }) => (
              <FormItem>
                <FormLabel>User Info URL</FormLabel>
                <FormControl>
                  <Input
                    placeholder='https://provider.com/api/userinfo'
                    {...field}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name='scopes'
            render={({ field }) => (
              <FormItem>
                <FormLabel>Scopes</FormLabel>
                <FormControl>
                  <Input
                    placeholder='openid email profile'
                    {...field}
                  />
                </FormControl>
                <FormDescription>
                  Space-separated list of OAuth scopes
                </FormDescription>
                {template.default_scopes.length > 0 && (
                  <div className='flex flex-wrap gap-1 pt-1'>
                    {template.default_scopes.map((scope) => (
                      <Badge key={scope} variant='secondary' className='text-xs'>
                        {scope}
                      </Badge>
                    ))}
                  </div>
                )}
                <FormMessage />
              </FormItem>
            )}
          />
        </CollapsibleContent>
      </Collapsible>
    </div>
  )
}
