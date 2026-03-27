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
      updates: Partial<Pick<BuilderNode, 'props' | 'styles' | 'content'>>,
    ) => {
      setTree(
        updateNodeInTree(tree, nodeId, (node) => ({
          ...node,
          ...(updates.props && { props: { ...node.props, ...updates.props } }),
          ...(updates.styles && {
            styles: { ...node.styles, ...updates.styles },
          }),
          ...(updates.content !== undefined && { content: updates.content }),
        })),
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
    <BuilderContext.Provider value={value}>{children}</BuilderContext.Provider>
  )
}

export function useBuilderContext(): BuilderContextValue {
  const context = useContext(BuilderContext)
  if (!context) {
    throw new Error('useBuilderContext must be used within a BuilderProvider')
  }
  return context
}
