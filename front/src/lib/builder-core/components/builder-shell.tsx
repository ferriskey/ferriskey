import {
  DndContext,
  PointerSensor,
  pointerWithin,
  useSensor,
  useSensors,
  type DragEndEvent,
  type DragOverEvent,
  type DragStartEvent,
} from '@dnd-kit/core'
import { createContext, useContext, useState, type ReactNode } from 'react'
import type { Active } from '@dnd-kit/core'
import { useBuilderContext } from '../context'
import type { BuilderNode } from '../types'
import { findNode } from '../utils'
import { BuilderDragOverlay } from './drag-overlay'

export interface DropIndicator {
  overId: string
  position: 'before' | 'after' | 'inside'
}

const DropIndicatorContext = createContext<DropIndicator | null>(null)

export function useDropIndicator() {
  return useContext(DropIndicatorContext)
}

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
  const [dropIndicator, setDropIndicator] = useState<DropIndicator | null>(null)

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

  function handleDragOver(event: DragOverEvent) {
    const { over } = event
    if (!over) {
      setDropIndicator(null)
      return
    }

    const overData = over.data.current

    if (overData?.parentId !== undefined) {
      // Over an empty drop zone or canvas root
      setDropIndicator({ overId: String(over.id), position: 'inside' })
    } else if (overData?.node) {
      // Over a sortable node — use the activatorEvent to determine before/after
      const overElement = document.querySelector(`[data-sortable-id="${over.id}"]`)
      if (overElement) {
        const rect = overElement.getBoundingClientRect()
        const pointerEvent = event.activatorEvent as PointerEvent
        const dragY = (event.delta?.y ?? 0) + (pointerEvent?.clientY ?? 0)
        const midY = rect.top + rect.height / 2
        setDropIndicator({
          overId: String(over.id),
          position: dragY < midY ? 'before' : 'after',
        })
      } else {
        setDropIndicator({ overId: String(over.id), position: 'after' })
      }
    } else {
      setDropIndicator(null)
    }
  }

  function handleDragEnd(event: DragEndEvent) {
    const currentIndicator = dropIndicator
    setActiveItem(null)
    setDropIndicator(null)

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
      // If the indicator says "after", insert after the hovered node
      if (currentIndicator?.overId === String(over.id) && currentIndicator.position === 'after') {
        targetIndex += 1
      }
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
      collisionDetection={pointerWithin}
      onDragStart={handleDragStart}
      onDragOver={handleDragOver}
      onDragEnd={handleDragEnd}
      onDragCancel={() => {
        setActiveItem(null)
        setDropIndicator(null)
      }}
    >
      <DropIndicatorContext.Provider value={dropIndicator}>
        {children}
      </DropIndicatorContext.Provider>
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
