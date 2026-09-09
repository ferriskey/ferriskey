import type { ReactNode } from 'react'
import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { PageTabs, Pill, Squircle, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import Client = Schemas.Client

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
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)

  const backButton = (
    <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
      <ArrowLeft className='size-3.5' />
      Clients
    </Button>
  )

  if (isLoading) {
    return (
      <div className={container}>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100' />
          </div>
        </div>
      </div>
    )
  }

  if (!client) {
    return (
      <div className={container}>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700'>Client not found</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500'>
            It may have been deleted, or it belongs to another realm.
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className={container}>
      {backButton}

      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div className='flex items-center gap-3'>
          <Squircle name={client.name || client.client_id} size='xl' />
          <div className='min-w-0'>
            <h1 className={tokens.header.title}>{client.name}</h1>
            <p className='font-mono-ui text-xs text-neutral-400'>{client.client_id}</p>
            <div className='mt-1.5 flex flex-wrap items-center gap-2'>
              <Pill tone={client.public_client ? 'info' : 'violet'} mono>
                {client.public_client ? 'public' : 'confidential'}
              </Pill>
              <Pill tone='primary' mono>
                {client.protocol}
              </Pill>
              <Pill tone={client.enabled ? 'success' : 'neutral'} mono>
                {client.enabled ? 'enabled' : 'disabled'}
              </Pill>
              {client.maintenance_enabled && <Pill tone='amber'>in maintenance</Pill>}
            </div>
          </div>
        </div>

        <dl className='shrink-0 text-right text-xs text-neutral-500'>
          <dt className='sr-only'>Created at</dt>
          <dd className='tnum'>Created {new Date(client.created_at).toLocaleDateString('en-GB')}</dd>
          <dt className='sr-only'>Identifier</dt>
          <dd className='font-mono-ui text-[11px] text-neutral-400'>{client.id}</dd>
        </dl>
      </div>

      <PageTabs tabs={tabs} value={tab} className='mt-5'>
        <div className={tokens.page.blockGap}>{children}</div>
      </PageTabs>
    </div>
  )
}
