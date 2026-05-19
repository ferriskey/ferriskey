import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from 'react'
import type {
  BuilderActions,
  BuilderAdapter,
  BuilderNode,
  BuilderState,
} from './types'
import { BreakpointProvider } from './breakpoint-context'
import {
  generateNodeId,
  insertNodeInTree,
  moveNodeInTree,
  removeNodeFromTree,
  updateNodeInTree,
} from './utils'

interface BuilderContextValue extends BuilderState, BuilderActions {
  adapter: BuilderAdapter
}

const BuilderContext = createContext<BuilderContextValue | null>(null)

interface BuilderProviderProps {
  adapter: BuilderAdapter
  initialTree?: BuilderNode[]
  children: ReactNode
  onChange?: (tree: BuilderNode[]) => void
}

export function BuilderProvider({
  adapter,
  initialTree = [],
  children,
  onChange,
}: BuilderProviderProps) {
  const [tree, setTreeState] = useState<BuilderNode[]>(initialTree)
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null)

  const setTree = useCallback(
    (newTree: BuilderNode[]) => {
      setTreeState(newTree)
      onChange?.(newTree)
    },
    [onChange],
  )

  const addNode = useCallback(
    (type: string, parentId: string | null, index?: number) => {
      const defaultNode = adapter.getDefaultNode(type)
      const node: BuilderNode = {
        ...defaultNode,
        id: generateNodeId(),
      }
      setTree(
        insertNodeInTree(
          tree,
          node,
          parentId,
          index ?? (parentId === null ? tree.length : Infinity),
        ),
      )
      setSelectedNodeId(node.id)
    },
    [adapter, tree, setTree],
  )

  const removeNode = useCallback(
    (nodeId: string) => {
      setTree(removeNodeFromTree(tree, nodeId))
      if (selectedNodeId === nodeId) {
        setSelectedNodeId(null)
      }
    },
    [tree, selectedNodeId, setTree],
  )

  const moveNode = useCallback(
    (nodeId: string, newParentId: string | null, newIndex: number) => {
      setTree(moveNodeInTree(tree, nodeId, newParentId, newIndex))
    },
    [tree, setTree],
  )

  const updateNode = useCallback(
    (
      nodeId: string,
      updates: Partial<
        Pick<BuilderNode, 'name' | 'props' | 'styles' | 'content' | 'breakpoints'>
      >,
    ) => {
      setTree(
        updateNodeInTree(tree, nodeId, (node) => {
          // Merge breakpoint overrides per-bp so callers send only the
          // changed key for the changed breakpoint.
          let nextBreakpoints = node.breakpoints
          if (updates.breakpoints) {
            const merged: Record<string, Record<string, unknown>> = {
              ...(node.breakpoints ?? {}),
            }
            for (const [bp, override] of Object.entries(updates.breakpoints)) {
              if (override === undefined) {
                delete merged[bp]
              } else {
                merged[bp] = {
                  ...(node.breakpoints?.[bp as 'sm' | 'md' | 'lg' | 'xl'] ?? {}),
                  ...override,
                }
                // Strip empty-string overrides so the resolver falls back to
                // the base value (treating "" as "no override").
                for (const k of Object.keys(merged[bp])) {
                  if (merged[bp][k] === '') delete merged[bp][k]
                }
                if (Object.keys(merged[bp]).length === 0) delete merged[bp]
              }
            }
            nextBreakpoints =
              Object.keys(merged).length > 0
                ? (merged as BuilderNode['breakpoints'])
                : undefined
          }
          return {
            ...node,
            ...(updates.name !== undefined && { name: updates.name }),
            ...(updates.props && { props: { ...node.props, ...updates.props } }),
            ...(updates.styles && {
              styles: { ...node.styles, ...updates.styles },
            }),
            ...(updates.content !== undefined && { content: updates.content }),
            ...(updates.breakpoints !== undefined && { breakpoints: nextBreakpoints }),
          }
        }),
      )
    },
    [tree, setTree],
  )

  const selectNode = useCallback((nodeId: string | null) => {
    setSelectedNodeId(nodeId)
  }, [])

  const value = useMemo<BuilderContextValue>(
    () => ({
      tree,
      selectedNodeId,
      adapter,
      addNode,
      removeNode,
      moveNode,
      updateNode,
      selectNode,
      setTree,
    }),
    [
      tree,
      selectedNodeId,
      adapter,
      addNode,
      removeNode,
      moveNode,
      updateNode,
      selectNode,
      setTree,
    ],
  )

  return (
    <BuilderContext.Provider value={value}>
      <BreakpointProvider>{children}</BreakpointProvider>
    </BuilderContext.Provider>
  )
}

export function useBuilderContext(): BuilderContextValue {
  const context = useContext(BuilderContext)
  if (!context) {
    throw new Error('useBuilderContext must be used within a BuilderProvider')
  }
  return context
}
