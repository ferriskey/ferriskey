import { useDroppable } from '@dnd-kit/core'
import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { useBuilderContext } from '../context'
import type { BuilderNode } from '../types'

interface SortableNodeProps {
  node: BuilderNode
  depth: number
}

function SortableNode({ node, depth }: SortableNodeProps) {
  const { selectedNodeId, selectNode, adapter } = useBuilderContext()
  const isSelected = selectedNodeId === node.id

  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
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

  return (
    <div ref={setNodeRef} style={style} {...attributes}>
      <div
        className={`group relative cursor-pointer rounded border transition-colors ${
          isSelected
            ? 'border-primary bg-primary/5'
            : 'border-transparent hover:border-border'
        }`}
        style={{ marginLeft: depth * 12 }}
        onClick={(e) => {
          e.stopPropagation()
          selectNode(node.id)
        }}
      >
        <div
          className='flex items-center gap-1.5 px-2 py-1 text-xs'
          {...listeners}
        >
          {componentDef?.icon && (
            <span className='flex h-4 w-4 shrink-0 items-center justify-center text-muted-foreground'>
              {componentDef.icon}
            </span>
          )}
          <span className='truncate font-medium'>
            {componentDef?.label ?? node.type}
          </span>
          {node.content && (
            <span className='ml-auto truncate text-muted-foreground max-w-[120px]'>
              {node.content.replace(/<[^>]*>/g, '').slice(0, 30)}
            </span>
          )}
        </div>

        {node.children.length > 0 && (
          <DroppableChildren depth={depth + 1}>
            {node.children}
          </DroppableChildren>
        )}

        {componentDef?.isContainer && node.children.length === 0 && (
          <EmptyDropZone parentId={node.id} depth={depth + 1} />
        )}
      </div>
    </div>
  )
}

interface DroppableChildrenProps {
  children: BuilderNode[]
  depth: number
}

function DroppableChildren({
  children,
  depth,
}: DroppableChildrenProps) {
  return (
    <SortableContext
      items={children.map((c) => c.id)}
      strategy={verticalListSortingStrategy}
    >
      <div className='py-0.5'>
        {children.map((child) => (
          <SortableNode key={child.id} node={child} depth={depth} />
        ))}
      </div>
    </SortableContext>
  )
}

function EmptyDropZone({
  parentId,
  depth,
}: {
  parentId: string
  depth: number
}) {
  const { setNodeRef, isOver } = useDroppable({
    id: `empty-${parentId}`,
    data: { parentId },
  })

  return (
    <div
      ref={setNodeRef}
      className={`mx-2 my-1 rounded border border-dashed py-3 text-center text-xs text-muted-foreground transition-colors ${
        isOver ? 'border-primary bg-primary/5' : 'border-border'
      }`}
      style={{ marginLeft: depth * 12 }}
    >
      Drop components here
    </div>
  )
}

export function Canvas() {
  const { tree, selectNode } = useBuilderContext()

  const { setNodeRef, isOver } = useDroppable({
    id: 'canvas-root',
    data: { parentId: null },
  })

  return (
    <div
      ref={setNodeRef}
      className={`min-h-[300px] flex-1 rounded-lg border p-2 transition-colors ${
        isOver ? 'border-primary bg-primary/5' : 'border-border bg-muted/30'
      }`}
      onClick={() => selectNode(null)}
    >
      {tree.length === 0 ? (
        <div className='flex h-full min-h-[280px] items-center justify-center text-sm text-muted-foreground'>
          Drag components here to start building
        </div>
      ) : (
        <SortableContext
          items={tree.map((n) => n.id)}
          strategy={verticalListSortingStrategy}
        >
          {tree.map((node) => (
            <SortableNode key={node.id} node={node} depth={0} />
          ))}
        </SortableContext>
      )}
    </div>
  )
}
