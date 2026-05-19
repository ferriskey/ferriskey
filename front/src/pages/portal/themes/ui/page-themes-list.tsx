import { useState } from 'react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { CheckCircle2, Palette, Pencil, Plus, Trash2 } from 'lucide-react'
import type { Schemas } from '@/api/api.client'

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
    <div className='flex flex-col gap-4 p-6'>
      <div className='flex items-center justify-between'>
        <div>
          <h2 className='text-2xl font-semibold'>Themes</h2>
          <p className='text-sm text-muted-foreground'>
            Customize the realm's authentication portal — colors, fonts, spacing, and the seven
            page trees rendered by the renderer.
          </p>
        </div>
        <Button onClick={() => setCreateOpen(true)}>
          <Plus size={16} />
          New theme
        </Button>
      </div>

      {isLoading ? (
        <div className='flex items-center justify-center py-12 text-sm text-muted-foreground'>
          Loading themes…
        </div>
      ) : themes.length === 0 ? (
        <Card>
          <CardContent className='flex flex-col items-center justify-center gap-3 py-12'>
            <Palette size={40} className='text-muted-foreground' />
            <p className='text-sm text-muted-foreground'>
              No themes yet. Create one to start customizing the portal.
            </p>
            <Button variant='outline' onClick={() => setCreateOpen(true)}>
              <Plus size={16} />
              Create theme
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className='grid gap-3'>
          {themes.map((theme) => {
            const isActive = activeThemeId === theme.id
            return (
              <Card key={theme.id}>
                <CardHeader className='flex flex-row items-center justify-between pb-2'>
                  <div className='flex items-center gap-3'>
                    <CardTitle className='text-base'>{theme.name}</CardTitle>
                    {isActive && (
                      <Badge variant='outline' className='gap-1'>
                        <CheckCircle2 size={12} />
                        Active
                      </Badge>
                    )}
                  </div>
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
                </CardHeader>
              </Card>
            )
          })}
        </div>
      )}

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
