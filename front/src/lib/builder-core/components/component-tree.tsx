import { useDndContext, useDraggable, useDroppable } from '@dnd-kit/core'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { Fragment, useMemo, useState } from 'react'
import { useBuilderContext } from '../context'
import type { BuilderAdapter, BuilderNode } from '../types'
import { useDropIndicator } from './builder-shell'

/**
 * Hierarchical view of the current builder tree — alternative to the
 * drag-and-drop palette for navigating deeply nested layouts. Clicking a
 * row selects the corresponding node in the same way as clicking it in the
 * canvas, so the config panel updates accordingly. Lets the author reach
 * blocks that are hidden (display:none at the active breakpoint) or sit
 * behind overlapping siblings in the canvas.
 *
 * Drag-and-drop mirrors Figma's Layers panel:
 *  - drag any row by its body (a 1px activation distance means a click
 *    without movement still selects, but starting to drag picks up the row)
 *  - hover the middle of a *container* row → drop "into" (append child)
 *  - hover the top or bottom of any row → drop above / below as a sibling,
 *    with a thin indicator line showing exactly where the block will land
 *
 * The shell's existing `handleDragOver` / `handleDragEnd` already know how
 * to treat `{ source: 'canvas', node }` actives and `{ node }` / `{ parentId,
 * position }` overs — the tree just becomes a second emitter of those
 * shapes.
 */
export function ComponentTree() {
  const { tree, selectedNodeId, selectNode, adapter } = useBuilderContext()

  if (tree.length === 0) {
    return (
      <div className='p-3 text-xs text-muted-foreground'>
        The tree is empty — drop a component into the canvas to get started.
      </div>
    )
  }

  return (
    <div className='flex flex-col gap-0.5 p-2'>
      {tree.map((node) => (
        <TreeRow
          key={node.id}
          node={node}
          depth={0}
          selectedNodeId={selectedNodeId}
          onSelect={selectNode}
          adapter={adapter}
        />
      ))}
    </div>
  )
}

interface TreeRowProps {
  node: BuilderNode
  depth: number
  selectedNodeId: string | null
  onSelect: (nodeId: string) => void
  adapter: BuilderAdapter
}

function TreeRow({ node, depth, selectedNodeId, onSelect, adapter }: TreeRowProps) {
  const [open, setOpen] = useState(true)
  const isSelected = selectedNodeId === node.id
  const hasChildren = node.children.length > 0

  // Look up the human-readable label from the adapter (e.g. "Heading", "Div")
  // so the tree row matches what the user sees in the components palette.
  const def = useMemo(
    () => adapter.components.find((c) => c.type === node.type),
    [adapter.components, node.type],
  )
  const fallbackLabel = def?.label ?? node.type
  const label = node.name?.trim() || fallbackLabel
  const showsCustomName = Boolean(node.name?.trim()) && label !== fallbackLabel

  // Drag source — emits `{ source: 'canvas', node }` so builder-shell's
  // existing canvas-move pipeline handles the drop without any new code.
  // `fromTree: true` is a hint the shell uses to skip the iframe-coordinate
  // translation: the tree row lives in the parent document, not in the
  // canvas iframe, so its pointer/rect are already in parent-window space.
  const { attributes, listeners, setNodeRef: setDragRef, isDragging } = useDraggable({
    id: `tree-${node.id}`,
    data: { source: 'canvas', fromTree: true, node },
  })

  const isContainer = def?.isContainer ?? false

  // Cycle guard: a row can't accept a drop that would make it a descendant
  // of itself. Walk the dragged node's subtree once per render — cheap for
  // the size of trees auth pages tend to ship (a few dozen blocks).
  const { active } = useDndContext()
  const draggedNode = (active?.data?.current as { node?: BuilderNode } | undefined)?.node
  const droppingIntoSelfOrDescendant =
    !!draggedNode &&
    (draggedNode.id === node.id || isDescendant(draggedNode, node.id))

  // Sibling drop target — covers the row body, lets the shell compute
  // before/after based on pointer Y vs the row's center. Provides the same
  // `{ node }` payload as the canvas's `useSortable`, so handleDragOver +
  // handleDragEnd treat tree drops identically to canvas reorders.
  const { setNodeRef: setSiblingDropRef } = useDroppable({
    id: `tree-row-${node.id}`,
    data: { node },
    disabled: droppingIntoSelfOrDescendant,
  })

  // "Into" drop target — overlays the middle band of the row and wins
  // collision when the cursor is centred (pointerWithin picks the smaller
  // overlapping droppable). Containers only — leaves can't host children.
  const { setNodeRef: setIntoDropRef, isOver: isOverInto } = useDroppable({
    id: `tree-into-${node.id}`,
    data: { parentId: node.id, position: 'append' },
    disabled: !isContainer || droppingIntoSelfOrDescendant,
  })

  // Read the shell's drop indicator to paint before/after lines exactly
  // where the canvas does — same overId convention (`String(over.id)`),
  // same before/after vocabulary.
  const indicator = useDropIndicator()
  const overId = `tree-row-${node.id}`
  const showBefore = indicator?.overId === overId && indicator.position === 'before'
  const showAfter = indicator?.overId === overId && indicator.position === 'after'

  return (
    <Fragment>
      {showBefore && <DropLine depth={depth} />}
      <div
        ref={setSiblingDropRef}
        className={`relative flex flex-col rounded-md ${
          isOverInto && isContainer ? 'bg-primary/10 ring-1 ring-primary/40' : ''
        }`}
      >
        <div
          ref={(el) => {
            setDragRef(el)
          }}
          {...listeners}
          {...attributes}
          className={`group flex h-7 items-center gap-1 rounded-md px-1 text-xs transition-colors ${
            isSelected
              ? 'bg-primary/10 text-primary'
              : 'text-foreground/80 hover:bg-muted'
          } ${isDragging ? 'opacity-50' : ''}`}
          // Indent per level so the hierarchy is visually obvious. Compact
          // enough that 6+ levels still fit in the narrow sidebar. Whole
          // row is grabbable (Figma-style); a 1px PointerSensor activation
          // distance keeps single clicks distinguishable from drags.
          style={{ paddingLeft: depth * 12 + 4, cursor: isDragging ? 'grabbing' : 'grab' }}
        >
          {hasChildren ? (
            <button
              type='button'
              onPointerDown={(e) => e.stopPropagation()}
              onClick={(e) => {
                e.stopPropagation()
                setOpen((v) => !v)
              }}
              className='flex h-4 w-4 shrink-0 items-center justify-center rounded text-muted-foreground hover:text-foreground'
              aria-label={open ? 'Collapse' : 'Expand'}
            >
              {open ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
            </button>
          ) : (
            // Same-width spacer so labels align across leaf / branch rows.
            <span className='h-4 w-4 shrink-0' />
          )}
          {def?.icon && (
            <span className='flex h-4 w-4 shrink-0 items-center justify-center text-muted-foreground'>
              {def.icon}
            </span>
          )}
          <button
            type='button'
            // Stop pointerdown so the drag listeners on the wrapper don't
            // consume the click — without this, a single tap on the label
            // would arm a drag (and the activation distance saves us 99%
            // of the time, but the cursor-over-label feedback is cleaner
            // when the label clearly behaves like a "select" affordance).
            onPointerDown={(e) => e.stopPropagation()}
            onClick={(e) => {
              e.stopPropagation()
              onSelect(node.id)
            }}
            className='flex min-w-0 flex-1 items-center gap-1.5 text-left'
          >
            <span className='truncate'>{label}</span>
            {showsCustomName && (
              <span className='shrink-0 text-[10px] uppercase tracking-wide text-muted-foreground'>
                {fallbackLabel}
              </span>
            )}
          </button>
        </div>
        {/* Centre overlay: middle ~50% of the row vertically. Wins
            collision when the cursor is roughly centred → drop "into" this
            container. Outside this band, the outer sibling droppable wins
            → before/after based on pointer Y vs row centre.
            `pointer-events` toggles based on whether a drag is in progress
            — otherwise the overlay swallows clicks on the label and the
            admin can't select the container from the tree. */}
        {isContainer && !droppingIntoSelfOrDescendant && (
          <div
            ref={setIntoDropRef}
            style={{
              position: 'absolute',
              left: 0,
              right: 0,
              top: 6,
              bottom: 6,
              pointerEvents: active ? 'auto' : 'none',
            }}
            aria-hidden
          />
        )}
      </div>
      {showAfter && <DropLine depth={depth} />}
      {open && hasChildren && (
        <div>
          {node.children.map((child) => (
            <TreeRow
              key={child.id}
              node={child}
              depth={depth + 1}
              selectedNodeId={selectedNodeId}
              onSelect={onSelect}
              adapter={adapter}
            />
          ))}
        </div>
      )}
    </Fragment>
  )
}

/**
 * Insertion indicator drawn between rows. `depth` matches the surrounding
 * rows so the line sits flush with the would-be sibling's left edge — same
 * visual cue Figma uses to disambiguate "insert as sibling at this depth"
 * from "insert into the parent above".
 */
function DropLine({ depth }: { depth: number }) {
  return (
    <div
      className='relative h-0.5'
      style={{ marginLeft: depth * 12 + 4 }}
      aria-hidden
    >
      <div className='absolute inset-x-0 top-1/2 h-0.5 -translate-y-1/2 rounded-full bg-primary shadow-[0_0_6px_rgba(99,93,255,0.4)]' />
    </div>
  )
}

/**
 * Returns `true` when `targetId` is anywhere in `node`'s descendant subtree.
 * Used to prevent the user from dropping a parent inside its own child,
 * which would make `moveNodeInTree` silently lose the move (the node gets
 * removed before the insertion target is found).
 */
function isDescendant(node: BuilderNode, targetId: string): boolean {
  for (const child of node.children) {
    if (child.id === targetId) return true
    if (isDescendant(child, targetId)) return true
  }
  return false
}
