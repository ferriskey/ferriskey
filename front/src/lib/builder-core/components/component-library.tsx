import { useDraggable } from '@dnd-kit/core'
import { useComponentLibrary } from '../hooks'
import type { ComponentDefinition } from '../types'

interface DraggableComponentProps {
  definition: ComponentDefinition
}

function DraggableComponent({ definition }: DraggableComponentProps) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: `library-${definition.type}`,
    data: {
      source: 'library',
      type: definition.type,
    },
  })

  return (
    <div
      ref={setNodeRef}
      {...listeners}
      {...attributes}
      className={`flex cursor-grab items-center gap-2 rounded-md border border-border bg-card p-2 text-sm transition-colors hover:bg-accent ${
        isDragging ? 'opacity-50' : ''
      }`}
    >
      {definition.icon && (
        <span className='flex h-5 w-5 shrink-0 items-center justify-center text-muted-foreground'>
          {definition.icon}
        </span>
      )}
      <span className='truncate'>{definition.label}</span>
    </div>
  )
}

export function ComponentLibrary() {
  const components = useComponentLibrary()

  return (
    <div className='flex flex-col gap-1.5 p-2'>
      <h3 className='px-1 text-xs font-medium uppercase tracking-wider text-muted-foreground'>
        Components
      </h3>
      <div className='flex flex-col gap-1'>
        {components.map((def) => (
          <DraggableComponent key={def.type} definition={def} />
        ))}
      </div>
    </div>
  )
}
