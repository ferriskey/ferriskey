import {
  DndContext,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
  type DragStartEvent,
} from '@dnd-kit/core'
import { useState, type ReactNode } from 'react'
import type { Active } from '@dnd-kit/core'
import { useBuilderContext } from '../context'
import type { BuilderNode } from '../types'
import { findNode } from '../utils'
import { BuilderDragOverlay } from './DragOverlay'

interface BuilderShellProps {
  children: ReactNode
}

/**
 * Wraps the builder UI with DndContext and handles drag events.
 * This is the main layout shell — the consumer provides children
 * (Canvas, ComponentLibrary, ConfigPanel) in whatever layout they want.
 */
export function BuilderShell({ children }: BuilderShellProps) {
  const { addNode, moveNode, tree } = useBuilderContext()
  const [activeItem, setActiveItem] = useState<Active | null>(null)

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 5,
      },
    }),
  )

  function handleDragStart(event: DragStartEvent) {
    setActiveItem(event.active)
  }

  function handleDragEnd(event: DragEndEvent) {
    setActiveItem(null)

    const { active, over } = event
    if (!over) return

    const activeData = active.data.current
    const overData = over.data.current

    // Determine the target parent
    let targetParentId: string | null = null
    let targetIndex = 0

    if (overData?.parentId !== undefined) {
      // Dropped on an empty zone or canvas root
      targetParentId = overData.parentId
      targetIndex = 0
    } else if (overData?.node) {
      // Dropped on another sortable node — insert as sibling
      const overNode = overData.node
      // Find parent of the over node
      targetParentId = findParentId(tree, overNode.id)
      const siblings =
        targetParentId === null
          ? tree
          : (findNode(tree, targetParentId)?.children ?? [])
      targetIndex = siblings.findIndex((n) => n.id === overNode.id)
      if (targetIndex < 0) targetIndex = siblings.length
    }

    if (activeData?.source === 'library') {
      // New component from library
      addNode(activeData.type, targetParentId, targetIndex)
    } else if (activeData?.source === 'canvas') {
      // Reorder existing node
      const nodeId = activeData.node?.id
      if (nodeId && nodeId !== over.id) {
        moveNode(nodeId, targetParentId, targetIndex)
      }
    }
  }

  return (
    <DndContext
      sensors={sensors}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
    >
      {children}
      <BuilderDragOverlay activeItem={activeItem} />
    </DndContext>
  )
}

/**
 * Find the parent id of a node in the tree. Returns null if at root.
 */
function findParentId(
  tree: BuilderNode[],
  nodeId: string,
  parentId: string | null = null,
): string | null {
  for (const node of tree) {
    if (node.id === nodeId) return parentId
    const found = findParentId(node.children, nodeId, node.id)
    if (found !== undefined && found !== null) return found
    // Check if it was found at this level
    if (node.children.some((c) => c.id === nodeId)) return node.id
  }
  return null
}
