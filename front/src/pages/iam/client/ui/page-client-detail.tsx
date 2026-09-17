import type { ReactNode } from 'react'
import { ArrowLeft } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { DetailHeader, PageShell, PageTabs, Pill, Squircle, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import { clientAuthenticationOf, clientStateOf } from '../client-choices'

import Client = Schemas.Client
import { formatDate } from '@/utils/format-date'

export interface PageClientDetailProps {
  client?: Client
  isLoading: boolean
  tab: string
  tabs: TabItem[]
  onBack: () => void
  children: ReactNode
}

export default function PageClientDetail({
  client,
  isLoading,
  tab,
  tabs,
  onBack,
  children,
}: PageClientDetailProps) {
  const { t } = useTranslation('client')

  const backButton = (
    <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
      <ArrowLeft className='size-3.5' />
      {t('detail.back')}
    </Button>
  )

  if (isLoading) {
    return (
      <PageShell>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </PageShell>
    )
  }

  if (!client) {
    return (
      <PageShell>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('detail.not_found.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('detail.not_found.hint')}
          </p>
        </div>
      </PageShell>
    )
  }

  return (
    <PageShell>
      <DetailHeader
        onBack={onBack}
        backLabel={t('detail.back')}
        icon={<Squircle name={client.name || client.client_id} size='xl' />}
        title={client.name}
        caption={
          <p className='font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>{client.client_id}</p>
        }
        pills={
          <>
            <Pill tone={client.public_client ? 'info' : 'violet'} mono>
              {t(`shared.authentication.${clientAuthenticationOf(client.public_client)}`)}
            </Pill>
            <Pill tone='primary' mono>
              {client.protocol}
            </Pill>
            <Pill tone={client.enabled ? 'success' : 'neutral'} mono>
              {t(`shared.state.${clientStateOf(client.enabled)}`)}
            </Pill>
            {client.maintenance_enabled && <Pill tone='amber'>{t('shared.in_maintenance')}</Pill>}
          </>
        }
        meta={
          <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
            <dt className='sr-only'>{t('detail.meta.created_label')}</dt>
            <dd className='tnum'>{t('detail.meta.created', { date: formatDate(client.created_at) })}</dd>
            <dt className='sr-only'>{t('detail.meta.identifier')}</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{client.id}</dd>
          </dl>
        }
      />

      <PageTabs tabs={tabs} value={tab} className='mt-5'>
        <div className={tokens.page.blockGap}>{children}</div>
      </PageTabs>
    </PageShell>
  )
}
