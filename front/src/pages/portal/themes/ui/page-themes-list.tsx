import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Skeleton } from '@/components/ui/skeleton'
import { CheckCircle2, Palette, Pencil, Plus, Trash2 } from 'lucide-react'
import type { Schemas } from '@/api/api.client'
import { PortalOverviewHeader } from '../../components/portal-overview-header'

interface Props {
  themes: Schemas.PortalTheme[]
  activeThemeId: string | null
  isLoading: boolean
  isCreating: boolean
  onCreate: (name: string) => void
  onEdit: (id: string) => void
  onActivate: (id: string) => void
  onDelete: (id: string) => void
}

function ThemeAvatar({ name }: { name: string }) {
  return (
    <div
      className='h-10 w-10 rounded-md flex items-center justify-center shrink-0'
      style={{ backgroundColor: '#F97316' }}
    >
      <span className='text-base font-bold text-white'>
        {name?.[0]?.toUpperCase() || 'T'}
      </span>
    </div>
  )
}

function ActiveBadge() {
  return (
    <span className='inline-flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-semibold border border-emerald-400/50 text-emerald-600 bg-emerald-50 dark:bg-emerald-500/10'>
      <CheckCircle2 className='h-3 w-3' />
      ACTIVE
    </span>
  )
}

export default function PageThemesList({
  themes,
  activeThemeId,
  isLoading,
  isCreating,
  onCreate,
  onEdit,
  onActivate,
  onDelete,
}: Props) {
  const [createOpen, setCreateOpen] = useState(false)
  const [newName, setNewName] = useState('')

  const submitCreate = () => {
    if (!newName.trim()) return
    onCreate(newName.trim())
    setNewName('')
    setCreateOpen(false)
  }

  return (
    <div className='flex flex-col gap-6 p-8'>
      <PortalOverviewHeader
        activeTab='themes'
        primaryAction={{ label: 'New Theme', onClick: () => setCreateOpen(true) }}
      />

      <div>
        <div className='flex items-center justify-between mb-3'>
          <h2 className='text-base font-semibold'>Themes ({themes.length})</h2>
        </div>

        <div className='-mx-8 border-t border-b overflow-hidden'>
          {isLoading ? (
            Array.from({ length: 3 }).map((_, i) => (
              <div
                key={i}
                className='flex items-center justify-between px-8 py-4 border-b last:border-b-0'
              >
                <div className='flex items-center gap-3'>
                  <Skeleton className='h-10 w-10 rounded-md' />
                  <div className='space-y-2'>
                    <Skeleton className='h-4 w-40' />
                    <Skeleton className='h-3 w-32' />
                  </div>
                </div>
                <Skeleton className='h-6 w-20 rounded-md' />
              </div>
            ))
          ) : themes.length === 0 ? (
            <div className='flex flex-col items-center justify-center gap-3 py-16'>
              <Palette size={40} className='text-muted-foreground' />
              <p className='text-sm text-muted-foreground'>
                No themes yet. Create one to start customizing the portal.
              </p>
              <Button variant='outline' onClick={() => setCreateOpen(true)}>
                <Plus size={16} />
                Create theme
              </Button>
            </div>
          ) : (
            themes.map((theme) => {
              const isActive = activeThemeId === theme.id
              return (
                <div
                  key={theme.id}
                  className='flex items-center justify-between px-8 py-4 border-b last:border-b-0 hover:bg-muted/40 transition-colors'
                >
                  <div className='flex items-center gap-4'>
                    <ThemeAvatar name={theme.name} />
                    <div>
                      <div className='flex items-center gap-2.5'>
                        <span className='text-base font-medium'>{theme.name}</span>
                      </div>
                      <div className='text-sm text-muted-foreground mt-0.5'>
                        theme_id: {theme.id}
                      </div>
                    </div>
                  </div>

                  <div className='flex items-center gap-3'>
                    {isActive && <ActiveBadge />}
                    <div className='flex items-center gap-1'>
                      {!isActive && (
                        <Button
                          variant='ghost'
                          size='icon'
                          title='Activate'
                          onClick={() => onActivate(theme.id)}
                        >
                          <CheckCircle2 size={14} />
                        </Button>
                      )}
                      <Button
                        variant='ghost'
                        size='icon'
                        title='Edit'
                        onClick={() => onEdit(theme.id)}
                      >
                        <Pencil size={14} />
                      </Button>
                      <Button
                        variant='ghost'
                        size='icon'
                        title={isActive ? 'Cannot delete the active theme' : 'Delete'}
                        disabled={isActive}
                        className='text-destructive hover:text-destructive disabled:text-muted-foreground'
                        onClick={() => onDelete(theme.id)}
                      >
                        <Trash2 size={14} />
                      </Button>
                    </div>
                  </div>
                </div>
              )
            })
          )}
        </div>
      </div>

      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create a portal theme</DialogTitle>
          </DialogHeader>
          <Input
            placeholder='Theme name'
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') submitCreate()
            }}
          />
          <DialogFooter>
            <Button variant='outline' onClick={() => setCreateOpen(false)}>
              Cancel
            </Button>
            <Button onClick={submitCreate} disabled={isCreating || !newName.trim()}>
              {isCreating ? 'Creating…' : 'Create'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
