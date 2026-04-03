import { cn } from '@/lib/utils'
import { useDroppable } from '@dnd-kit/core'
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
      <DroppableChildren>{node.children}</DroppableChildren>
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
        className={cn('relative cursor-pointer', isSelected && 'z-10')}
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
}

function DroppableChildren({ children }: DroppableChildrenProps) {
  return (
    <SortableContext items={children.map((c) => c.id)} strategy={verticalListSortingStrategy}>
      {children.map((child) => (
        <SortableNode key={child.id} node={child} />
      ))}
    </SortableContext>
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
      className={`my-1 rounded border border-dashed py-4 text-center text-xs text-muted-foreground transition-colors ${
        isOver ? 'border-primary bg-primary/5' : 'border-border/50'
      }`}
    >
      Drop components here
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
      className='flex flex-1 justify-center p-6'
      style={{
        backgroundColor: '#f8f9fa',
        backgroundImage: 'radial-gradient(circle, #d1d5db 1px, transparent 1px)',
        backgroundSize: '20px 20px',
      }}
      onClick={() => selectNode(null)}
    >
      <div
        ref={setNodeRef}
        className={`min-h-[400px] rounded-lg bg-white shadow-sm transition-all duration-200 ${
          isOver ? 'ring-2 ring-primary ring-dashed' : ''
        }`}
        style={{ width: '100%', maxWidth }}
      >
        {tree.length === 0 ? (
          <div className='flex h-full min-h-[400px] items-center justify-center text-sm text-muted-foreground'>
            Drag components here to start building
          </div>
        ) : (
          <SortableContext items={tree.map((n) => n.id)} strategy={verticalListSortingStrategy}>
            {tree.map((node) => (
              <SortableNode key={node.id} node={node} />
            ))}
          </SortableContext>
        )}
      </div>
    </div>
  )
}
