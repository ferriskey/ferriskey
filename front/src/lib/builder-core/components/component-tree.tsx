import { ChevronDown, ChevronRight } from 'lucide-react'
import { useMemo, useState } from 'react'
import { useBuilderContext } from '../context'
import type { BuilderAdapter, BuilderNode } from '../types'

/**
 * Hierarchical view of the current builder tree — alternative to the
 * drag-and-drop palette for navigating deeply nested layouts. Clicking a
 * row selects the corresponding node in the same way as clicking it in the
 * canvas, so the config panel updates accordingly. Lets the author reach
 * blocks that are hidden (display:none at the active breakpoint) or sit
 * behind overlapping siblings in the canvas.
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

  return (
    <div className='flex flex-col'>
      <div
        className={`group flex h-7 items-center gap-1 rounded-md px-1 text-xs transition-colors ${
          isSelected
            ? 'bg-primary/10 text-primary'
            : 'text-foreground/80 hover:bg-muted'
        }`}
        // Indent per level so the hierarchy is visually obvious. Compact
        // enough that 6+ levels still fit in the narrow sidebar.
        style={{ paddingLeft: depth * 12 + 4 }}
      >
        {hasChildren ? (
          <button
            type='button'
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
    </div>
  )
}
