import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { LayoutTemplate, Pencil, Plus, Star, Trash2 } from 'lucide-react'

interface PortalLayoutListItem {
  id: string
  name: string
  is_default: boolean
  created_at: string
  updated_at: string
}

interface Props {
  layouts: PortalLayoutListItem[]
  isLoading: boolean
  onEdit: (id: string) => void
  onDelete: (id: string) => void
  onSetDefault: (id: string) => void
  onCreate: () => void
}

export default function PagePortalLayoutsList({
  layouts,
  isLoading,
  onEdit,
  onDelete,
  onSetDefault,
  onCreate,
}: Props) {
  return (
    <div className='flex flex-col gap-4 p-6'>
      <div className='flex items-center justify-between'>
        <div>
          <h1 className='text-2xl font-semibold'>Portal Layouts</h1>
          <p className='text-sm text-muted-foreground'>
            Design the layout shared across your authentication portal pages
          </p>
        </div>
        <Button onClick={onCreate}>
          <Plus size={16} />
          New Layout
        </Button>
      </div>

      {isLoading ? (
        <div className='flex items-center justify-center py-12 text-sm text-muted-foreground'>
          Loading layouts...
        </div>
      ) : layouts.length === 0 ? (
        <Card>
          <CardContent className='flex flex-col items-center justify-center gap-3 py-12'>
            <LayoutTemplate size={40} className='text-muted-foreground' />
            <p className='text-sm text-muted-foreground'>
              No portal layouts yet. Create one to get started.
            </p>
            <Button variant='outline' onClick={onCreate}>
              <Plus size={16} />
              Create Layout
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className='grid gap-3'>
          {layouts.map((layout) => (
            <Card key={layout.id}>
              <CardHeader className='flex flex-row items-center justify-between pb-2'>
                <div className='flex items-center gap-3'>
                  <CardTitle className='text-base'>{layout.name}</CardTitle>
                  {layout.is_default && (
                    <Badge variant='outline' className='gap-1'>
                      <Star size={12} />
                      Default
                    </Badge>
                  )}
                </div>
                <div className='flex items-center gap-1'>
                  {!layout.is_default && (
                    <Button
                      variant='ghost'
                      size='icon'
                      title='Set as default'
                      onClick={() => onSetDefault(layout.id)}
                    >
                      <Star size={14} />
                    </Button>
                  )}
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
              </CardHeader>
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}
