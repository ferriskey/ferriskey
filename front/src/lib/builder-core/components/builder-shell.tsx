import {
  DndContext,
  PointerSensor,
  pointerWithin,
  useSensor,
  useSensors,
  type CollisionDetection,
  type DragEndEvent,
  type DragOverEvent,
  type DragStartEvent,
} from '@dnd-kit/core'
import { createContext, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
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
  /**
   * Bounding rect of an iframe whose document hosts droppables registered
   * with this DndContext. When set, collision detection translates every
   * iframe-inside droppable rect into parent-window coordinates so dnd-kit
   * can match it against the parent's pointer events. Pass `null` (default)
   * when there is no iframe canvas.
   */
  getIframeRect?: () => DOMRect | null
  /**
   * Current visual scale factor applied to the iframe element (e.g. via
   * `transform: scale(...)`). Droppable rects reported from inside the iframe
   * are in unscaled iframe-document coordinates, so the collision check
   * multiplies them by this scale before adding the iframe's parent-window
   * offset. Defaults to 1 (no scaling).
   */
  getIframeScale?: () => number
}

/**
 * Wraps the builder UI with DndContext and handles drag events.
 * This is the main layout shell — the consumer provides children
 * (Canvas, ComponentLibrary, ConfigPanel) in whatever layout they want.
 */
export function BuilderShell({
  children,
  getIframeRect,
  getIframeScale,
}: BuilderShellProps) {
  const { addNode, moveNode, tree } = useBuilderContext()
  const [activeItem, setActiveItem] = useState<Active | null>(null)
  const [dropIndicator, setDropIndicator] = useState<DropIndicator | null>(null)
  const getIframeRectRef = useRef(getIframeRect)
  const getIframeScaleRef = useRef(getIframeScale)
  useEffect(() => {
    getIframeRectRef.current = getIframeRect
  }, [getIframeRect])
  useEffect(() => {
    getIframeScaleRef.current = getIframeScale
  }, [getIframeScale])

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 5,
      },
    }),
  )

  /**
   * Collision detection that's aware of a child iframe.
   *
   * When the canvas is rendered inside an iframe, droppables registered
   * inside it report their bounding rects in iframe-local coordinates. The
   * pointer events captured by the parent's PointerSensor (the iframe sets
   * `pointer-events: none` during drag) are in parent-window coordinates.
   * Without translation these would never overlap. We offset every iframe-
   * inside droppable's rect by the iframe's position in the parent so both
   * sides agree on a single coordinate space.
   */
  const collisionDetection = useMemo<CollisionDetection>(
    () => (args) => {
      const offset = getIframeRectRef.current?.()
      if (!offset) return pointerWithin(args)
      const scale = getIframeScaleRef.current?.() ?? 1

      const adjusted: typeof args.droppableRects = new Map()
      args.droppableRects.forEach((rect, id) => {
        const container = args.droppableContainers.find((c) => c.id === id)
        const el = container?.node?.current as HTMLElement | null | undefined
        const isInIframe = el?.ownerDocument && el.ownerDocument !== document
        if (isInIframe) {
          // Inner rect lives in unscaled iframe-document coords; multiply by
          // the iframe's visual scale before adding the parent-window offset
          // so pointer/rect overlap is computed in a single coordinate space.
          adjusted.set(id, {
            top: rect.top * scale + offset.top,
            bottom: rect.bottom * scale + offset.top,
            left: rect.left * scale + offset.left,
            right: rect.right * scale + offset.left,
            width: rect.width * scale,
            height: rect.height * scale,
          })
        } else {
          adjusted.set(id, rect)
        }
      })

      return pointerWithin({ ...args, droppableRects: adjusted })
    },
    [],
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
      // Dropped on an empty zone, a container's trailing "+" zone, or canvas
      // root — append to the end of that parent's children. Appending is the
      // intuitive default; without it, dropping on a flex/grid silently
      // re-prepended to the realm root which surprised everyone.
      targetParentId = overData.parentId
      const siblings =
        targetParentId === null
          ? tree
          : (findNode(tree, targetParentId)?.children ?? [])
      targetIndex = siblings.length
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
      collisionDetection={collisionDetection}
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
