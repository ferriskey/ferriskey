import { DragOverlay as DndDragOverlay } from '@dnd-kit/core'
import type { Active } from '@dnd-kit/core'
import { useBuilderContext } from '../context'

interface BuilderDragOverlayProps {
  activeItem: Active | null
}

export function BuilderDragOverlay({ activeItem }: BuilderDragOverlayProps) {
  const { adapter } = useBuilderContext()

  if (!activeItem) return null

  const data = activeItem.data.current
  const type =
    data?.source === 'library'
      ? data.type
      : data?.source === 'canvas'
        ? data.node?.type
        : null

  if (!type) return null

  const def = adapter.components.find((c) => c.type === type)

  return (
    <DndDragOverlay>
      <div className='flex items-center gap-2 rounded-md border border-primary bg-card px-3 py-2 text-sm shadow-lg'>
        {def?.icon && (
          <span className='flex h-4 w-4 items-center justify-center text-muted-foreground'>
            {def.icon}
          </span>
        )}
        <span>{def?.label ?? type}</span>
      </div>
    </DndDragOverlay>
  )
}
