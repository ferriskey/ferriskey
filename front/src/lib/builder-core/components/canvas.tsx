import { useDndContext, useDroppable } from '@dnd-kit/core'
import { SortableContext, useSortable, verticalListSortingStrategy } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { useBuilderContext } from '../context'
import type { BuilderNode } from '../types'
import { findNode } from '../utils'
import { useDropIndicator } from './builder-shell'

function DropIndicatorLine() {
  return (
    <div className='relative flex items-center py-1'>
      <div className='bg-primary ring-primary/30 h-2.5 w-2.5 rounded-full ring-4' />
      <div className='bg-primary shadow-primary/40 h-1 flex-1 rounded-full shadow-[0_0_8px]' />
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

  // Mirror item-positioning props (currently `order`) on the wrapper that
  // dnd-kit registers as the real flex/grid item. Without this, the user's
  // base `order` value lives on the inner block (which isn't a grid item)
  // and has no visual effect in the editor — even though it works fine at
  // runtime where there's no wrapper. Per-breakpoint overrides flow through
  // the `[data-sortable-id]` selector in `generateBreakpointCss`.
  const orderProp = (node.props.order as string) || ''
  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : 1,
    order: orderProp ? Number(orderProp) : undefined,
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
      <PrependDropZone parentId={parentId} />
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
      className={`m-1 flex min-h-[56px] items-center justify-center rounded-md border-2 border-dashed py-4 text-center text-xs font-medium transition-all ${
        isOver
          ? 'border-primary bg-primary/10 text-primary ring-primary/20 scale-[1.01] ring-4'
          : 'border-border/70 text-muted-foreground/80'
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
/**
 * Resolve a human label for a drop zone targeting `parentId`. `null` means
 * the page tree itself (the user navigates to this via the "Root" crumb);
 * stacked zones use distinct labels so a user staring at two adjacent
 * "insert at top" bars can tell which level (page vs. container) they'd
 * actually land in.
 */
function useParentLabel(parentId: string | null): string {
  const { tree, adapter } = useBuilderContext()
  if (parentId === null) return 'Root'
  const parent = findNode(tree, parentId)
  if (!parent) return 'parent'
  const componentDef = adapter.components.find((c) => c.type === parent.type)
  return parent.name || componentDef?.label || parent.type
}

function AppendDropZone({ parentId }: { parentId: string | null }) {
  const { active } = useDndContext()
  const { setNodeRef, isOver } = useDroppable({
    id: parentId === null ? 'append-root' : `append-${parentId}`,
    data: { parentId, position: 'append' },
  })
  const parentLabel = useParentLabel(parentId)

  // No drag in progress → render nothing so the zone doesn't participate in
  // the parent's flex/grid/block flow.
  if (!active) return null

  return (
    <div
      ref={setNodeRef}
      className={`mt-1 flex items-center justify-center rounded-md border-2 border-dashed text-center text-xs font-medium transition-all ${
        isOver
          ? 'border-primary bg-primary/10 ring-primary/20 min-h-[48px] py-4 text-primary opacity-100 ring-4'
          : 'border-border/60 min-h-[32px] py-3 text-muted-foreground/70'
      }`}
    >
      + Append to end of {parentLabel}
    </div>
  )
}

/**
 * Leading target rendered before each container's existing children. Mirrors
 * AppendDropZone — same "only during drag" rule so it doesn't perturb the
 * parent's flex/grid flow when the user isn't actively dragging. Without
 * this, inserting at index 0 requires the user to precisely hover the first
 * sortable's top half, which is unforgiving when that sortable is small or
 * lives inside a large empty container.
 */
function PrependDropZone({ parentId }: { parentId: string | null }) {
  const { active } = useDndContext()
  const { setNodeRef, isOver } = useDroppable({
    id: parentId === null ? 'prepend-root' : `prepend-${parentId}`,
    data: { parentId, position: 'prepend' },
  })
  const parentLabel = useParentLabel(parentId)

  if (!active) return null

  return (
    <div
      ref={setNodeRef}
      className={`mb-1 flex items-center justify-center rounded-md border-2 border-dashed text-center text-xs font-medium transition-all ${
        isOver
          ? 'border-primary bg-primary/10 ring-primary/20 min-h-[48px] py-4 text-primary opacity-100 ring-4'
          : 'border-border/60 min-h-[32px] py-3 text-muted-foreground/70'
      }`}
    >
      + Insert at start of {parentLabel}
    </div>
  )
}

interface CanvasProps {
  maxWidth?: number
  /**
   * When `true` (default), the canvas stretches to fill its parent's full
   * height — the right behavior for a stand-alone editor where the iframe
   * IS the canvas. Set to `false` when the canvas is mounted inside a
   * layout's `<page-content>` slot: the layout already controls vertical
   * alignment (e.g. centering the form), so the canvas should behave like
   * a block of its content height — matching how `<form>` renders at
   * runtime — and let the layout's centering apply.
   */
  fillHeight?: boolean
}

export function Canvas({ maxWidth = 600, fillHeight = true }: CanvasProps) {
  const { tree, selectNode } = useBuilderContext()

  const { setNodeRef, isOver } = useDroppable({
    id: 'canvas-root',
    data: { parentId: null },
  })

  const emptyState = (
    <div
      ref={setNodeRef}
      className={`transition-all duration-200 ${
        isOver ? 'ring-2 ring-primary ring-dashed' : ''
      } ${fillHeight ? 'min-h-screen' : ''}`}
      style={{ width: '100%', maxWidth }}
    >
      {tree.length === 0 ? (
        <div
          className={`flex items-center justify-center text-sm text-muted-foreground ${
            fillHeight ? 'h-full min-h-screen' : 'py-8'
          }`}
        >
          Drag components here to start building
        </div>
      ) : (
        <DroppableChildren parentId={null}>{tree}</DroppableChildren>
      )}
    </div>
  )

  if (!fillHeight) {
    // Inline mode: render the drop zone as a block so the surrounding layout
    // can position it (the layout's `<page-content>` div is the flex item).
    return <div onClick={() => selectNode(null)}>{emptyState}</div>
  }

  return (
    <div
      className='flex min-h-full flex-1 items-start justify-center p-6'
      onClick={() => selectNode(null)}
    >
      {emptyState}
    </div>
  )
}
