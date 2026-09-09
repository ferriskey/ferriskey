import type { ReactNode } from 'react'
import { useRef } from 'react'
import { Plus, Upload } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { PageTabs, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

export interface PageEmailsProps {
  tab: string
  tabs: TabItem[]
  onCreate: () => void
  onImport: (file: File) => void
  children: ReactNode
}

export default function PageEmails({ tab, tabs, onCreate, onImport, children }: PageEmailsProps) {
  const importInput = useRef<HTMLInputElement>(null)

  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <div className={cn('flex flex-wrap items-start justify-between gap-3', tokens.header.spacing)}>
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>Emails</h1>
          <p className='mt-0.5 text-sm text-neutral-500'>
            Models of the transactional emails of this realm, and the server that sends them.
          </p>
        </div>
        {tab === 'templates' && (
          <div className='flex shrink-0 gap-2'>
            <Button variant='outline' onClick={() => importInput.current?.click()}>
              <Upload /> Import
            </Button>
            <input
              ref={importInput}
              type='file'
              accept='application/json,.json'
              className='hidden'
              onChange={(event) => {
                const file = event.target.files?.[0]
                if (file) onImport(file)
                event.target.value = ''
              }}
            />
            <Button onClick={onCreate}>
              <Plus /> New template
            </Button>
          </div>
        )}
      </div>

      <PageTabs tabs={tabs} value={tab}>
        <div className='mt-4'>{children}</div>
      </PageTabs>
    </div>
  )
}
