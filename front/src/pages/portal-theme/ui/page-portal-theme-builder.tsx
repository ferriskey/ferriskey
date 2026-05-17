import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { usePortalThemeContext } from '../context/portal-theme-context'
import { BordersPanel } from '../components/panels/borders-panel'
import { ColorsPanel } from '../components/panels/colors-panel'
import { FontsPanel } from '../components/panels/fonts-panel'
import { PreviewCard } from '../components/preview-card'
import { SidebarNav } from '../components/sidebar-nav'

type Props = {
  isSaving: boolean
  onSave: () => void
}

export default function PagePortalThemeBuilder({ isSaving, onSave }: Props) {
  const { activeTab, isDirty, discard } = usePortalThemeContext()

  return (
    <div className='flex h-[calc(100vh-3rem)] flex-col'>
      <header className='flex items-center justify-between border-b border-border px-6 py-3'>
        <div>
          <h1 className='text-lg font-semibold'>Portal theme</h1>
          <p className='text-xs text-muted-foreground'>
            {isDirty ? 'Unsaved changes' : 'All changes are live'}
          </p>
        </div>
        <div className='flex items-center gap-2'>
          <Button variant='outline' size='sm' onClick={discard} disabled={!isDirty || isSaving}>
            Discard
          </Button>
          <Button size='sm' onClick={onSave} disabled={!isDirty || isSaving}>
            {isSaving ? 'Saving…' : 'Save and publish'}
          </Button>
        </div>
      </header>

      <div className='grid flex-1 grid-cols-[200px_1fr_360px] overflow-hidden'>
        <aside className='border-r border-border'>
          <SidebarNav />
        </aside>

        <main className='overflow-hidden bg-muted/30'>
          <PreviewCard />
        </main>

        <aside className='border-l border-border'>
          <ScrollArea className='h-full'>
            <div className='p-4'>
              {activeTab === 'colors' && <ColorsPanel />}
              {activeTab === 'fonts' && <FontsPanel />}
              {activeTab === 'borders' && <BordersPanel />}
            </div>
          </ScrollArea>
        </aside>
      </div>
    </div>
  )
}
