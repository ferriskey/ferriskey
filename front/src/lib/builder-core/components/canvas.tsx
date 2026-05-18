import { useDndContext, useDroppable } from '@dnd-kit/core'
import { SortableContext, useSortable, verticalListSortingStrategy } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { useBuilderContext } from '../context'
import type { BuilderNode } from '../types'
import { useDropIndicator } from './builder-shell'

function DropIndicatorLine() {
  return (
    <div className='relative flex items-center py-0.5'>
      <div className='bg-primary h-2 w-2 rounded-full' />
      <div className='bg-primary h-0.5 flex-1 rounded-full' />
    </div>
  )
}

interface SortableNodeProps {
  node: BuilderNode
}

function SortableNode({ node }: SortableNodeProps) {
  const { selectedNodeId, selectNode, adapter } = useBuilderContext()
  const isSelected = selectedNodeId === node.id
  const indicator = useDropIndicator()

  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: node.id,
    data: {
      source: 'canvas',
      node,
    },
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : 1,
  }

  const componentDef = adapter.components.find((c) => c.type === node.type)

  const renderedChildren =
    node.children.length > 0 ? (
      <DroppableChildren parentId={node.id}>{node.children}</DroppableChildren>
    ) : componentDef?.isContainer ? (
      <EmptyDropZone parentId={node.id} />
    ) : null

  const visualBlock = adapter.renderVisualBlock
    ? adapter.renderVisualBlock(node, isSelected, renderedChildren)
    : null

  const showBefore = indicator?.overId === node.id && indicator.position === 'before'
  const showAfter = indicator?.overId === node.id && indicator.position === 'after'

  return (
    <div ref={setNodeRef} style={style} data-sortable-id={node.id} {...attributes}>
      {showBefore && <DropIndicatorLine />}
      <div
        // No `position: relative` here — it would establish a containing
        // block, trapping every `position: absolute` descendant of the user's
        // tree inside this tiny click wrapper instead of letting it resolve
        // against the iframe viewport like it does in the live portal.
        className='cursor-pointer'
        onClick={(e) => {
          e.stopPropagation()
          selectNode(node.id)
        }}
        {...listeners}
      >
        {visualBlock ?? (
          <FallbackNode node={node} isSelected={isSelected}>
            {renderedChildren}
          </FallbackNode>
        )}
      </div>
      {showAfter && <DropIndicatorLine />}
    </div>
  )
}

function FallbackNode({
  node,
  isSelected,
  children,
}: {
  node: BuilderNode
  isSelected: boolean
  children?: React.ReactNode
}) {
  const { adapter } = useBuilderContext()
  const componentDef = adapter.components.find((c) => c.type === node.type)

  return (
    <div
      className={`rounded border p-2 text-xs transition-colors ${
        isSelected ? 'border-primary bg-primary/5' : 'border-transparent hover:border-border'
      }`}
    >
      <div className='flex items-center gap-1.5'>
        {componentDef?.icon && (
          <span className='flex h-4 w-4 shrink-0 items-center justify-center text-muted-foreground'>
            {componentDef.icon}
          </span>
        )}
        <span className='truncate font-medium'>{componentDef?.label ?? node.type}</span>
      </div>
      {children}
    </div>
  )
}

interface DroppableChildrenProps {
  children: BuilderNode[]
  /** null = canvas root. Used by the trailing zone to know where to append. */
  parentId: string | null
}

function DroppableChildren({ children, parentId }: DroppableChildrenProps) {
  return (
    <>
      <SortableContext items={children.map((c) => c.id)} strategy={verticalListSortingStrategy}>
        {children.map((child) => (
          <SortableNode key={child.id} node={child} />
        ))}
      </SortableContext>
      <AppendDropZone parentId={parentId} />
    </>
  )
}

function EmptyDropZone({ parentId }: { parentId: string }) {
  const { setNodeRef, isOver } = useDroppable({
    id: `empty-${parentId}`,
    data: { parentId },
  })

  return (
    <div
      ref={setNodeRef}
      className={`m-1 rounded border border-dashed py-3 text-center text-[11px] text-muted-foreground/70 transition-colors ${
        isOver ? 'border-primary bg-primary/5 text-primary' : 'border-border/60'
      }`}
    >
      Drop components here
    </div>
  )
}

/**
 * Trailing target rendered after each container's existing children (and at
 * the end of the canvas root). Only visible while a drag is in progress —
 * otherwise it would occupy a flex/grid slot in the parent and skew the
 * positioning of the user's real blocks (e.g., a centered slot would no
 * longer sit at the visual center because this zone takes up the second
 * slot in the centering group).
 */
function AppendDropZone({ parentId }: { parentId: string | null }) {
  const { active } = useDndContext()
  const { setNodeRef, isOver } = useDroppable({
    id: parentId === null ? 'append-root' : `append-${parentId}`,
    data: { parentId },
  })

  // No drag in progress → render nothing so the zone doesn't participate in
  // the parent's flex/grid/block flow.
  if (!active) return null

  return (
    <div
      ref={setNodeRef}
      className={`mt-1 rounded border border-dashed text-center text-[11px] transition-all ${
        isOver
          ? 'border-primary bg-primary/5 py-3 text-primary opacity-100'
          : 'border-border/40 py-2 text-muted-foreground/60'
      }`}
    >
      + Drop here to append
    </div>
  )
}

interface CanvasProps {
  maxWidth?: number
}

export function Canvas({ maxWidth = 600 }: CanvasProps) {
  const { tree, selectNode } = useBuilderContext()

  const { setNodeRef, isOver } = useDroppable({
    id: 'canvas-root',
    data: { parentId: null },
  })

  return (
    <div
      className='flex min-h-full flex-1 items-start justify-center'
      onClick={() => selectNode(null)}
    >
      <div
        ref={setNodeRef}
        className={`min-h-screen transition-all duration-200 ${
          isOver ? 'ring-2 ring-primary ring-dashed' : ''
        }`}
        style={{ width: '100%', maxWidth }}
      >
        {tree.length === 0 ? (
          <div className='flex h-full min-h-screen items-center justify-center text-sm text-muted-foreground'>
            Drag components here to start building
          </div>
        ) : (
          <DroppableChildren parentId={null}>{tree}</DroppableChildren>
        )}
      </div>
    </div>
  )
}
