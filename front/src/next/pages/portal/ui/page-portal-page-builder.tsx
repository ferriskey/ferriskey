import type { CSSProperties } from 'react'
import { Link } from 'react-router-dom'
import { ArrowLeft, Save } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Pill } from '@/components/kit'
import { cn } from '@/lib/utils'
import type { BuilderNode } from '@/lib/builder-core'
import PageTreeEditor from '@/pages/portal/themes/components/page-tree-editor'
import { PORTAL_PAGES, labelForPortalPage, type PortalPageType } from '../portal-pages'

export interface PagePortalPageBuilderProps {
  realm: string
  themeName: string
  pageType: PortalPageType
  initialTree: unknown
  layoutTree: BuilderNode[] | null
  layoutName?: string
  cssVars: CSSProperties
  isDirty: boolean
  isSaving: boolean
  pageHref: (pageType: string) => string
  onTreeChange: (tree: BuilderNode[]) => void
  onBack: () => void
  onSave: () => void
}

export default function PagePortalPageBuilder({
  realm,
  themeName,
  pageType,
  initialTree,
  layoutTree,
  layoutName,
  cssVars,
  isDirty,
  isSaving,
  pageHref,
  onTreeChange,
  onBack,
  onSave,
}: PagePortalPageBuilderProps) {
  const nav = (
    <nav className='flex flex-col gap-0.5 p-2'>
      <p className='px-2 pb-1 text-[11px] font-medium uppercase tracking-wide text-neutral-400 dark:text-neutral-500'>
        Pages
      </p>
      {PORTAL_PAGES.map((page) => (
        <Link
          key={page.type}
          to={pageHref(page.type)}
          className={cn(
            'rounded-md px-2 py-1.5 text-[13px] transition-colors',
            page.type === pageType
              ? 'bg-fk-primary-soft font-medium text-fk-primary-text'
              : 'text-neutral-600 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-800'
          )}
        >
          {page.label}
        </Link>
      ))}
    </nav>
  )

  return (
    <div className='flex h-full min-h-0 w-full min-w-0 flex-col overflow-hidden'>
      <header className='flex shrink-0 flex-wrap items-center gap-3 border-b border-fk-line px-5 py-2.5'>
        <Button
          variant='ghost'
          size='sm'
          className='-ml-2 text-neutral-500 dark:text-neutral-400'
          onClick={onBack}
        >
          <ArrowLeft className='size-3.5' />
          {themeName}
        </Button>

        <div className='flex min-w-0 flex-1 flex-wrap items-center gap-2'>
          <h1 className='truncate text-[13px] font-semibold text-neutral-900 dark:text-neutral-100'>
            {labelForPortalPage(pageType)} page
          </h1>
          <Pill tone='neutral' mono>
            {pageType}
          </Pill>
          <Pill tone={layoutName ? 'violet' : 'neutral'}>
            {layoutName ? `framed by ${layoutName}` : 'no layout'}
          </Pill>
          {isDirty && <Pill tone='amber'>unsaved</Pill>}
        </div>

        <Button onClick={onSave} disabled={isSaving || !isDirty}>
          <Save className='size-4' />
          {isSaving ? 'Saving…' : 'Save page'}
        </Button>
      </header>

      <div className='min-h-0 flex-1 overflow-hidden'>
        <PageTreeEditor
          key={pageType}
          realm={realm}
          pageType={pageType}
          initialTree={initialTree}
          layoutTree={layoutTree}
          cssVars={cssVars}
          onTreeChange={onTreeChange}
          leftRailNav={nav}
        />
      </div>
    </div>
  )
}
