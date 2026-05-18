import { useDraggable } from '@dnd-kit/core'
import {
  LAYOUT_ONLY_BLOCK_TYPES,
  REQUIRED_BLOCK_TYPES,
  portalComponents,
} from '@/lib/builder-portal'
import type { ComponentDefinition } from '@/lib/builder-core'

interface Props {
  /** Block types the API requires for the currently-edited page. */
  requiredTypes: string[]
}

export function PageComponentLibrary({ requiredTypes }: Props) {
  const generic = portalComponents.filter(
    (c) => !REQUIRED_BLOCK_TYPES.has(c.type) && !LAYOUT_ONLY_BLOCK_TYPES.has(c.type),
  )
  const required = requiredTypes
    .map((type) => portalComponents.find((c) => c.type === type))
    .filter((c): c is ComponentDefinition => Boolean(c))

  return (
    <div className='flex flex-col gap-4 p-2'>
      <Section title='Components'>
        {generic.map((def) => (
          <DraggableComponent key={def.type} definition={def} />
        ))}
      </Section>
      {required.length > 0 && (
        <Section title='Required for this page'>
          {required.map((def) => (
            <DraggableComponent key={def.type} definition={def} />
          ))}
        </Section>
      )}
    </div>
  )
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className='flex flex-col gap-1.5'>
      <h3 className='px-1 text-xs font-medium uppercase tracking-wider text-muted-foreground'>
        {title}
      </h3>
      <div className='flex flex-col gap-1'>{children}</div>
    </div>
  )
}

function DraggableComponent({ definition }: { definition: ComponentDefinition }) {
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
