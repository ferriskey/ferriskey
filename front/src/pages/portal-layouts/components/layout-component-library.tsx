import { useDraggable } from '@dnd-kit/core'
import {
  LAYOUT_ONLY_BLOCK_TYPES,
  REQUIRED_BLOCK_TYPES,
  portalComponents,
} from '@/lib/builder-portal'
import type { ComponentDefinition } from '@/lib/builder-core'

export function LayoutComponentLibrary() {
  // Layouts only host generic decoration blocks. The page-specific required
  // blocks (email_input, password_input, totp_input, submit_button) live in
  // page trees, never in a layout, so we strip them out here. `page-content`
  // is the one "required" block for a layout — it marks where the page tree
  // will be slotted in at render time.
  const generic = portalComponents.filter(
    (c) => !REQUIRED_BLOCK_TYPES.has(c.type) && !LAYOUT_ONLY_BLOCK_TYPES.has(c.type),
  )
  const required = portalComponents.filter((c) => LAYOUT_ONLY_BLOCK_TYPES.has(c.type))

  return (
    <div className='flex flex-col gap-4 p-2'>
      <Section title='Components'>
        {generic.map((def) => (
          <DraggableComponent key={def.type} definition={def} />
        ))}
      </Section>
      {required.length > 0 && (
        <Section title='Required for this layout'>
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
