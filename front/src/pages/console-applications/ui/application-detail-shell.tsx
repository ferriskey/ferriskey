import { Schemas } from '@/api/api.client'
import { ArrowLeft, ShieldOff } from 'lucide-react'
import { APPLICATION_TONE, getApplicationTypeMeta, inferApplicationType } from '../types'

import Client = Schemas.Client

export type AppTab =
  | 'quickstart'
  | 'settings'
  | 'credentials'
  | 'api-access'
  | 'connections'
  | 'login-experience'
  | 'addons'
  | 'maintenance'

export interface TabDef {
  id: AppTab
  label: string
  /** Disabled tabs render a "Soon" badge and can't be selected. */
  soon?: boolean
}

interface Props {
  client: Client
  tabs: TabDef[]
  activeTab: AppTab
  onSelectTab: (tab: AppTab) => void
  onBack: () => void
  children: React.ReactNode
}

export default function ApplicationDetailShell({
  client,
  tabs,
  activeTab,
  onSelectTab,
  onBack,
  children,
}: Props) {
  const meta = getApplicationTypeMeta(inferApplicationType(client))
  const tone = APPLICATION_TONE[meta.tone]

  return (
    <div className='flex flex-col'>
      {/* Header */}
      <div className='px-8 md:px-12 pt-8 md:pt-12'>
        <button
          type='button'
          onClick={onBack}
          className='inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors mb-4'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Back to applications
        </button>
        <div className='flex items-start gap-4'>
          <div className={`h-12 w-12 rounded-md flex items-center justify-center shrink-0 ${tone.bg}`}>
            <meta.icon className={`h-5 w-5 ${tone.fg}`} />
          </div>
          <div className='flex-1 min-w-0'>
            <div className='flex items-center gap-2'>
              <h1 className='text-2xl font-medium tracking-tight truncate'>
                {client.name || client.client_id}
              </h1>
              <span className={`inline-flex items-center rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide ${tone.bg} ${tone.fg}`}>
                {meta.short}
              </span>
              {!client.enabled && (
                <span className='inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-medium bg-muted text-muted-foreground border border-border uppercase tracking-wide'>
                  <ShieldOff className='h-2.5 w-2.5' />
                  Off
                </span>
              )}
            </div>
            <p className='text-xs text-muted-foreground font-mono mt-1'>{client.client_id}</p>
          </div>
        </div>
      </div>

      {/* Tab nav */}
      <div className='mt-6 px-8 md:px-12 border-b border-border'>
        <nav className='flex items-center gap-1 overflow-x-auto'>
          {tabs.map((tab) => {
            const active = tab.id === activeTab
            return (
              <button
                key={tab.id}
                type='button'
                disabled={tab.soon}
                onClick={() => !tab.soon && onSelectTab(tab.id)}
                className={`relative inline-flex items-center gap-1.5 whitespace-nowrap px-3 py-2.5 text-sm font-medium transition-colors -mb-px border-b-2 ${
                  active
                    ? 'border-primary text-foreground'
                    : tab.soon
                      ? 'border-transparent text-muted-foreground/50 cursor-not-allowed'
                      : 'border-transparent text-muted-foreground hover:text-foreground'
                }`}
              >
                {tab.label}
                {tab.soon && (
                  <span className='rounded-md bg-muted px-1.5 py-0.5 text-[9px] font-semibold uppercase tracking-wide text-muted-foreground'>
                    Soon
                  </span>
                )}
              </button>
            )
          })}
        </nav>
      </div>

      {/* Tab body */}
      <div className='p-8 md:p-12 max-w-3xl'>{children}</div>
    </div>
  )
}
