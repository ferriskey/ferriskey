import { ExternalLink, Info } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Section } from '@/components/kit'
import ProviderIcon from '@/components/provider-icon'
import {
  templateDisplayName,
  type ProviderTemplate,
} from '@/constants/identity-provider-templates'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import CopyValue from './copy-value'

export interface ProviderSetupRailProps {
  template: ProviderTemplate
  callbackUrl: string
}

const CUSTOM_TEMPLATE_ID = 'custom'

const CUSTOM_STEP_KEYS = [
  'setup.steps.custom.console',
  'setup.steps.custom.application',
  'setup.steps.custom.credentials',
  'setup.steps.custom.redirect_uri',
  'setup.steps.custom.urls',
] as const

const TEMPLATE_STEP_KEYS = [
  'setup.steps.template.console',
  'setup.steps.template.application',
  'setup.steps.template.redirect_uri',
  'setup.steps.template.credentials',
  'setup.steps.template.paste',
] as const

export default function ProviderSetupRail({ template, callbackUrl }: ProviderSetupRailProps) {
  const { t } = useTranslation('identity-provider')

  const isCustom = template.id === CUSTOM_TEMPLATE_ID
  const providerLabel = templateDisplayName(template)
  const stepKeys = isCustom ? CUSTOM_STEP_KEYS : TEMPLATE_STEP_KEYS

  return (
    <aside className='lg:sticky lg:top-4 lg:self-start'>
      <Section
        title={t('setup.title')}
        description={t('setup.description')}
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
                  {providerLabel}
                </p>
                <p className='truncate text-[11px] uppercase tracking-wide text-neutral-400 dark:text-neutral-500'>
                  {t('setup.provider_subtitle', { type: template.provider_type })}
                </p>
              </div>
            </div>

            {template.documentation_url && (
              <Button variant='outline' size='sm' className='w-full' asChild>
                <a href={template.documentation_url} target='_blank' rel='noreferrer'>
                  {t('setup.documentation')}
                  <ExternalLink />
                </a>
              </Button>
            )}

            <div className='mt-3'>
              <p className='pb-1 text-[11px] font-medium text-neutral-700 dark:text-neutral-300'>
                {t('redirect_uri.label')}
              </p>
              <CopyValue value={callbackUrl} copyLabel={t('redirect_uri.copy')} />
              <p className='mt-1.5 text-[11px] leading-snug text-neutral-400 dark:text-neutral-500'>
                {t('setup.redirect_uri_hint')}
              </p>
            </div>

            <div className='mt-4 border-t border-fk-line-soft pt-3'>
              <p className='pb-1.5 text-[11px] font-medium uppercase tracking-wide text-neutral-500 dark:text-neutral-400'>
                {t('setup.steps.title')}
              </p>
              <ol className='space-y-1.5'>
                {stepKeys.map((stepKey, index) => (
                  <li key={stepKey} className='flex gap-2 text-[11px]'>
                    <span className='tnum shrink-0 text-neutral-400 dark:text-neutral-500'>
                      {index + 1}.
                    </span>
                    <span className='text-neutral-600 dark:text-neutral-400'>
                      {t(stepKey, { provider: providerLabel })}
                    </span>
                  </li>
                ))}
              </ol>
            </div>

            <div className='mt-3 flex items-start gap-2 rounded-md border border-fk-info-border bg-fk-info-soft/40 px-2.5 py-2'>
              <Info className='mt-0.5 size-3.5 shrink-0 text-fk-info' strokeWidth={2} />
              <p className='text-[11px] leading-snug text-neutral-600 dark:text-neutral-400'>
                {isCustom
                  ? t('setup.note.custom')
                  : t('setup.note.template', { provider: providerLabel })}
              </p>
            </div>

            <div className='mt-4 border-t border-fk-line-soft pt-3'>
              <p className='pb-1.5 text-[11px] font-medium uppercase tracking-wide text-neutral-500 dark:text-neutral-400'>
                {t('setup.scopes.title')}
              </p>
              {template.default_scopes.length > 0 ? (
                <>
                  <code className='block rounded-md border border-fk-line bg-neutral-50 px-2 py-1.5 font-mono-ui text-[11px] text-neutral-700 dark:bg-fk-surface dark:text-neutral-300'>
                    {template.default_scopes.join(' ')}
                  </code>
                  <p className='mt-1.5 text-[11px] leading-snug text-neutral-400 dark:text-neutral-500'>
                    {t('setup.scopes.hint')}
                  </p>
                </>
              ) : (
                <p className='text-[11px] leading-snug text-neutral-400 dark:text-neutral-500'>
                  {t('setup.scopes.empty')}
                </p>
              )}
            </div>
          </section>

          <div className='flex items-start gap-2 rounded-sm border border-fk-line px-3 py-2.5'>
            <Info className='mt-0.5 size-3.5 shrink-0 text-neutral-400 dark:text-neutral-500' strokeWidth={2} />
            <p className='text-[11px] leading-snug text-neutral-500 dark:text-neutral-400'>
              {t('setup.secret_note')}
            </p>
          </div>
        </div>
      </Section>
    </aside>
  )
}
