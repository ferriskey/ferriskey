import { ExternalLink, Info } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Section } from '@/components/kit'
import ProviderIcon from '@/components/provider-icon'
import type { ProviderTemplate } from '@/constants/identity-provider-templates'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import CopyValue from './copy-value'

export interface ProviderSetupRailProps {
  template: ProviderTemplate
  callbackUrl: string
}

const setupSteps = (template: ProviderTemplate) =>
  template.id === 'custom'
    ? [
        'Open the developer console of your OAuth provider',
        'Create a new OAuth application',
        'Copy the Client ID and the Client Secret',
        'Add the Redirect URI below to the application',
        'Fill in the authorization and token URLs',
      ]
    : [
        `Open the ${template.displayName} developer console`,
        'Create a new OAuth application',
        'Add the Redirect URI below',
        'Copy your Client ID and Client Secret',
        'Paste them into the form',
      ]

export default function ProviderSetupRail({ template, callbackUrl }: ProviderSetupRailProps) {
  return (
    <aside className='lg:sticky lg:top-4 lg:self-start'>
      <Section
        title='Preparation guide'
        description='What has to exist on the provider side.'
        contained={false}
      >
        <div className='space-y-3'>
          <section className={cn(tokens.surface.panel, 'p-3')}>
            <div className='flex items-center gap-2.5 pb-3'>
              <span className='grid size-9 shrink-0 place-items-center rounded-md border border-fk-line bg-white dark:bg-fk-surface'>
                <ProviderIcon icon={template.icon} size='sm' />
              </span>
              <div className='min-w-0'>
                <p className='truncate text-xs font-medium text-neutral-900 dark:text-neutral-100'>
                  {template.displayName}
                </p>
                <p className='truncate text-[11px] uppercase tracking-wide text-neutral-400 dark:text-neutral-500'>
                  {template.provider_type} provider
                </p>
              </div>
            </div>

            {template.documentation_url && (
              <Button variant='outline' size='sm' className='w-full' asChild>
                <a href={template.documentation_url} target='_blank' rel='noreferrer'>
                  View setup guide
                  <ExternalLink />
                </a>
              </Button>
            )}

            <div className='mt-3'>
              <p className='pb-1 text-[11px] font-medium text-neutral-700 dark:text-neutral-300'>Redirect URI</p>
              <CopyValue value={callbackUrl} label='the redirect URI' />
              <p className='mt-1.5 text-[11px] leading-snug text-neutral-400 dark:text-neutral-500'>
                Declare it as-is in the OAuth application. It changes if you change the alias.
              </p>
            </div>

            <div className='mt-4 border-t border-fk-line-soft pt-3'>
              <p className='pb-1.5 text-[11px] font-medium uppercase tracking-wide text-neutral-500 dark:text-neutral-400'>
                Setup steps
              </p>
              <ol className='space-y-1.5'>
                {setupSteps(template).map((step, index) => (
                  <li key={step} className='flex gap-2 text-[11px]'>
                    <span className='tnum shrink-0 text-neutral-400 dark:text-neutral-500'>{index + 1}.</span>
                    <span className='text-neutral-600 dark:text-neutral-400'>{step}</span>
                  </li>
                ))}
              </ol>
            </div>

            <div className='mt-3 flex items-start gap-2 rounded-md border border-fk-info-border bg-fk-info-soft/40 px-2.5 py-2'>
              <Info className='mt-0.5 size-3.5 shrink-0 text-fk-info' strokeWidth={2} />
              <p className='text-[11px] leading-snug text-neutral-600 dark:text-neutral-400'>
                {template.id === 'custom'
                  ? 'Nothing is pre-filled for a custom provider: the authorization and token URLs are yours to enter.'
                  : `The OAuth URLs are already filled in for ${template.displayName}. Only the credentials from its console are missing.`}
              </p>
            </div>

            <div className='mt-4 border-t border-fk-line-soft pt-3'>
              <p className='pb-1.5 text-[11px] font-medium uppercase tracking-wide text-neutral-500 dark:text-neutral-400'>
                Default scopes
              </p>
              {template.default_scopes.length > 0 ? (
                <>
                  <code className='block rounded-md border border-fk-line bg-neutral-50 px-2 py-1.5 font-mono-ui text-[11px] text-neutral-700 dark:bg-fk-surface dark:text-neutral-300'>
                    {template.default_scopes.join(' ')}
                  </code>
                  <p className='mt-1.5 text-[11px] leading-snug text-neutral-400 dark:text-neutral-500'>
                    Requested at every authorization. Editable in the form.
                  </p>
                </>
              ) : (
                <p className='text-[11px] leading-snug text-neutral-400 dark:text-neutral-500'>
                  None — this template declares no default scope.
                </p>
              )}
            </div>
          </section>

          <div className='flex items-start gap-2 rounded-sm border border-fk-line px-3 py-2.5'>
            <Info className='mt-0.5 size-3.5 shrink-0 text-neutral-400 dark:text-neutral-500' strokeWidth={2} />
            <p className='text-[11px] leading-snug text-neutral-500 dark:text-neutral-400'>
              The secret is encrypted on save and never readable afterwards — only replacing it
              is possible.
            </p>
          </div>
        </div>
      </Section>
    </aside>
  )
}
