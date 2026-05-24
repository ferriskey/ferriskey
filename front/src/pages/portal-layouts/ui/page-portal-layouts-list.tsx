import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { LayoutTemplate, Pencil, Plus, Trash2 } from 'lucide-react'
import { PortalOverviewHeader } from '@/pages/portal/components/portal-overview-header'

interface PortalLayoutListItem {
  id: string
  name: string
  created_at: string
  updated_at: string
}

interface Props {
  layouts: PortalLayoutListItem[]
  isLoading: boolean
  onEdit: (id: string) => void
  onDelete: (id: string) => void
  onCreate: () => void
}

function LayoutAvatar({ name }: { name: string }) {
  return (
    <div
      className='h-10 w-10 rounded-md flex items-center justify-center shrink-0'
      style={{ backgroundColor: '#F97316' }}
    >
      <span className='text-base font-bold text-white'>
        {name?.[0]?.toUpperCase() || 'L'}
      </span>
    </div>
  )
}

export default function PagePortalLayoutsList({
  layouts,
  isLoading,
  onEdit,
  onDelete,
  onCreate,
}: Props) {
  return (
    <div className='flex flex-col gap-6 p-8'>
      <PortalOverviewHeader
        activeTab='layouts'
        primaryAction={{ label: 'New Layout', onClick: onCreate }}
      />

      <div>
        <div className='flex items-center justify-between mb-3'>
          <h2 className='text-base font-semibold'>Layouts ({layouts.length})</h2>
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
          ) : layouts.length === 0 ? (
            <div className='flex flex-col items-center justify-center gap-3 py-16'>
              <LayoutTemplate size={40} className='text-muted-foreground' />
              <p className='text-sm text-muted-foreground'>
                No portal layouts yet. Create one to get started.
              </p>
              <Button variant='outline' onClick={onCreate}>
                <Plus size={16} />
                Create layout
              </Button>
            </div>
          ) : (
            layouts.map((layout) => (
              <div
                key={layout.id}
                className='flex items-center justify-between px-8 py-4 border-b last:border-b-0 hover:bg-muted/40 transition-colors'
              >
                <div className='flex items-center gap-4'>
                  <LayoutAvatar name={layout.name} />
                  <div>
                    <div className='flex items-center gap-2.5'>
                      <span className='text-base font-medium'>{layout.name}</span>
                    </div>
                    <div className='text-sm text-muted-foreground mt-0.5'>
                      layout_id: {layout.id}
                    </div>
                  </div>
                </div>

                <div className='flex items-center gap-1'>
                  <Button
                    variant='ghost'
                    size='icon'
                    title='Edit'
                    onClick={() => onEdit(layout.id)}
                  >
                    <Pencil size={14} />
                  </Button>
                  <Button
                    variant='ghost'
                    size='icon'
                    title='Delete'
                    className='text-destructive hover:text-destructive'
                    onClick={() => onDelete(layout.id)}
                  >
                    <Trash2 size={14} />
                  </Button>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  )
}
