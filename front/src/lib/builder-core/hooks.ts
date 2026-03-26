import { useMemo } from 'react'
import { useBuilderContext } from './context'
import { findNode } from './utils'

/**
 * Primary hook for accessing builder state and actions.
 */
export function useBuilder() {
  const ctx = useBuilderContext()
  return ctx
}

/**
 * Hook to get the currently selected node (or null).
 */
export function useSelectedNode() {
  const { tree, selectedNodeId } = useBuilderContext()

  return useMemo(() => {
    if (!selectedNodeId) return null
    return findNode(tree, selectedNodeId)
  }, [tree, selectedNodeId])
}

/**
 * Hook to get the list of available components from the adapter.
 */
export function useComponentLibrary() {
  const { adapter } = useBuilderContext()
  return adapter.components
}
