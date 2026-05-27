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
      // 1px is the smallest distance that still keeps clicks (which select a
      // node via onClick) distinguishable from drags. Larger values make the
      // drag overlay visibly lag the cursor by exactly that distance,
      // because `activatorEvent` points at pointerdown while dnd-kit's
      // transform tracks from the moment the threshold is crossed.
      activationConstraint: {
        distance: 1,
      },
    }),
  )

  /**
   * Collision detection that's aware of a child iframe.
   *
   * Droppables registered inside the canvas iframe report their bounding
   * rects in iframe-local coordinates; droppables registered in the parent
   * document are in parent-window coords. The pointer can be in either
   * coordinate system depending on where the drag started:
   *  - Library drag: pointerdown happens in the parent document, so dnd-kit
   *    records the pointer in parent-window coords.
   *  - Canvas drag: pointerdown happens in the iframe and dnd-kit's
   *    `setPointerCapture` keeps subsequent moves dispatching inside the
   *    iframe, so the pointer stays in iframe-local coords for the whole
   *    drag.
   *
   * We always normalise iframe-inside droppable rects to parent-window
   * coords (multiply by the iframe's visual scale, then offset by its parent
   * position). When the active source is itself iframe-internal, we apply
   * the same transformation to `pointerCoordinates` so pointer and rects
   * meet in a single coord space. Without this, the user has to push the
   * cursor right/down by exactly the iframe's offset to trigger a drop.
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

      // `Active.data.current.source === 'canvas'` is set by every `useSortable`
      // call inside the iframe (see `SortableNode` in `canvas.tsx`). We can't
      // rely on `args.active.node` because dnd-kit's public `Active` type
      // doesn't expose a node ref, so the previous `ownerDocument` check was
      // silently always false.
      const activeInIframe =
        (args.active?.data?.current as { source?: string } | undefined)?.source ===
        'canvas'
      const pointerCoordinates =
        activeInIframe && args.pointerCoordinates
          ? {
              x: args.pointerCoordinates.x * scale + offset.left,
              y: args.pointerCoordinates.y * scale + offset.top,
            }
          : args.pointerCoordinates

      return pointerWithin({
        ...args,
        droppableRects: adjusted,
        pointerCoordinates,
      })
    },
    [],
  )

  function handleDragStart(event: DragStartEvent) {
    setActiveItem(event.active)
  }

  function handleDragOver(event: DragOverEvent) {
    const { over, active } = event
    if (!over) {
      setDropIndicator(null)
      return
    }

    const overData = over.data.current

    if (overData?.parentId !== undefined) {
      // Over an empty drop zone or canvas root
      setDropIndicator({ overId: String(over.id), position: 'inside' })
    } else if (overData?.node) {
      // Use dnd-kit's translated rects so this works identically whether the
      // hovered sortable lives in the parent document or inside the iframe
      // canvas — `over.rect` is already in the collision-detection coordinate
      // space, and `active.rect.current.translated` follows the pointer as it
      // moves. Comparing the active block's center against the hovered
      // sibling's center yields a stable before/after flip without ever
      // double-counting the pointer offset.
      const overRect = over.rect
      const activeRect = active.rect.current.translated ?? active.rect.current.initial
      if (overRect && activeRect) {
        const activeCenterY = activeRect.top + activeRect.height / 2
        const overCenterY = overRect.top + overRect.height / 2
        setDropIndicator({
          overId: String(over.id),
          position: activeCenterY < overCenterY ? 'before' : 'after',
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
      // Dropped on an empty zone, a container's leading/trailing zone, or
      // canvas root. The `position` field distinguishes prepend vs append;
      // empty containers and the canvas-root fallback default to append (the
      // historical, less-surprising behavior for an empty parent).
      targetParentId = overData.parentId
      const siblings =
        targetParentId === null
          ? tree
          : (findNode(tree, targetParentId)?.children ?? [])
      targetIndex = overData.position === 'prepend' ? 0 : siblings.length
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
