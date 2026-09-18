import type { ReactNode } from 'react'
import { useRef } from 'react'
import { Plus, Upload } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { PageShell, PageTabs, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { EMAIL_TEMPLATE_NAMESPACE } from '../email-types'

const IMPORT_ACCEPT = 'application/json,.json'

export interface PageEmailsProps {
  tab: string
  tabs: TabItem[]
  onCreate: () => void
  onImport: (file: File) => void
  children: ReactNode
}

export default function PageEmails({ tab, tabs, onCreate, onImport, children }: PageEmailsProps) {
  const { t } = useTranslation(EMAIL_TEMPLATE_NAMESPACE)
  const importInput = useRef<HTMLInputElement>(null)

  return (
    <PageShell>
      <div className={cn('flex flex-wrap items-start justify-between gap-3', tokens.header.spacing)}>
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>{t('page.title')}</h1>
          <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
            {t('page.description')}
          </p>
        </div>
        {tab === 'templates' && (
          <div className='flex shrink-0 gap-2'>
            <Button variant='outline' onClick={() => importInput.current?.click()}>
              <Upload /> {t('page.import')}
            </Button>
            <input
              ref={importInput}
              type='file'
              accept={IMPORT_ACCEPT}
              className='hidden'
              onChange={(event) => {
                const file = event.target.files?.[0]
                if (file) onImport(file)
                event.target.value = ''
              }}
            />
            <Button onClick={onCreate}>
              <Plus /> {t('page.create')}
            </Button>
          </div>
        )}
      </div>

      <PageTabs tabs={tabs} value={tab}>
        <div className='mt-4'>{children}</div>
      </PageTabs>
    </PageShell>
  )
}
