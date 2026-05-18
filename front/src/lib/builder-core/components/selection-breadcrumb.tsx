import { ChevronRight, Layers } from 'lucide-react'
import { useEffect, useMemo } from 'react'
import { useBuilderContext } from '../context'
import { findNodePath } from '../utils'

/**
 * Compact breadcrumb of the selected node's ancestor chain.
 *
 * Solves the "I can't click my parent because the child fills it" problem:
 * each crumb is clickable and selects that ancestor. Also handles `Escape`
 * to walk up to the parent (or clear selection at the root).
 */
export function SelectionBreadcrumb() {
  const { tree, selectedNodeId, selectNode, adapter } = useBuilderContext()

  const path = useMemo(
    () => (selectedNodeId ? findNodePath(tree, selectedNodeId) : []),
    [tree, selectedNodeId],
  )

  // Escape → select parent, or deselect when at the root.
  useEffect(() => {
    if (!selectedNodeId) return
    const handler = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return
      e.preventDefault()
      if (path.length <= 1) {
        selectNode(null)
      } else {
        selectNode(path[path.length - 2].id)
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [selectedNodeId, path, selectNode])

  return (
    <div className='flex items-center gap-1 overflow-x-auto px-3 py-1.5 text-xs text-muted-foreground'>
      <button
        type='button'
        onClick={() => selectNode(null)}
        className={`flex items-center gap-1 rounded px-1.5 py-0.5 transition-colors hover:bg-muted ${
          path.length === 0 ? 'text-foreground' : ''
        }`}
        title='Deselect'
      >
        <Layers size={12} />
        <span>Root</span>
      </button>
      {path.map((node, idx) => {
        const def = adapter.components.find((c) => c.type === node.type)
        const isLast = idx === path.length - 1
        return (
          <span key={node.id} className='flex items-center gap-1'>
            <ChevronRight size={12} className='shrink-0 opacity-50' />
            <button
              type='button'
              onClick={() => selectNode(node.id)}
              className={`flex items-center gap-1 rounded px-1.5 py-0.5 transition-colors hover:bg-muted ${
                isLast ? 'bg-muted/60 text-foreground' : ''
              }`}
            >
              {def?.icon && <span className='flex h-3 w-3 items-center justify-center'>{def.icon}</span>}
              <span>{node.name?.trim() || def?.label || node.type}</span>
            </button>
          </span>
        )
      })}
      {selectedNodeId && (
        <span className='ml-2 hidden text-[10px] uppercase tracking-wider text-muted-foreground/60 sm:inline'>
          Esc to go up
        </span>
      )}
    </div>
  )
}
