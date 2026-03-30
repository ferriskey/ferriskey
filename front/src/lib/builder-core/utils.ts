import type { BuilderNode } from './types'

let counter = 0

export function generateNodeId(): string {
  counter += 1
  return `node-${Date.now()}-${counter}`
}

/**
 * Find a node by id in a tree, returning the node and its parent path.
 */
export function findNode(
  tree: BuilderNode[],
  nodeId: string,
): BuilderNode | null {
  for (const node of tree) {
    if (node.id === nodeId) return node
    const found = findNode(node.children, nodeId)
    if (found) return found
  }
  return null
}

/**
 * Deep clone a tree, applying a transform to a specific node.
 */
export function updateNodeInTree(
  tree: BuilderNode[],
  nodeId: string,
  updater: (node: BuilderNode) => BuilderNode,
): BuilderNode[] {
  return tree.map((node) => {
    if (node.id === nodeId) return updater(node)
    return {
      ...node,
      children: updateNodeInTree(node.children, nodeId, updater),
    }
  })
}

/**
 * Remove a node from the tree by id.
 */
export function removeNodeFromTree(
  tree: BuilderNode[],
  nodeId: string,
): BuilderNode[] {
  return tree
    .filter((node) => node.id !== nodeId)
    .map((node) => ({
      ...node,
      children: removeNodeFromTree(node.children, nodeId),
    }))
}

/**
 * Insert a node into the tree at a specific parent and index.
 * If parentId is null, insert at root level.
 */
export function insertNodeInTree(
  tree: BuilderNode[],
  node: BuilderNode,
  parentId: string | null,
  index: number,
): BuilderNode[] {
  if (parentId === null) {
    const newTree = [...tree]
    newTree.splice(index, 0, node)
    return newTree
  }

  return tree.map((n) => {
    if (n.id === parentId) {
      const newChildren = [...n.children]
      newChildren.splice(index, 0, node)
      return { ...n, children: newChildren }
    }
    return {
      ...n,
      children: insertNodeInTree(n.children, node, parentId, index),
    }
  })
}

/**
 * Move a node from its current position to a new parent/index.
 */
/**
 * Recursively regenerate all node IDs in a tree to avoid collisions.
 */
export function regenerateIds(nodes: BuilderNode[]): BuilderNode[] {
  return nodes.map((node) => ({
    ...node,
    id: generateNodeId(),
    children: regenerateIds(node.children),
  }))
}

export function moveNodeInTree(
  tree: BuilderNode[],
  nodeId: string,
  newParentId: string | null,
  newIndex: number,
): BuilderNode[] {
  const node = findNode(tree, nodeId)
  if (!node) return tree

  const withoutNode = removeNodeFromTree(tree, nodeId)
  return insertNodeInTree(withoutNode, node, newParentId, newIndex)
}
