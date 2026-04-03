import type { ReactNode } from 'react'

/**
 * A node in the builder tree. Renderer-agnostic.
 * The `type` field is opaque to the core — only the adapter knows what types exist.
 */
export interface BuilderNode {
  id: string
  type: string
  props: Record<string, unknown>
  styles: Record<string, unknown>
  children: BuilderNode[]
  content?: string
}

/**
 * Describes a component available in the builder library.
 * Provided by the adapter (renderer), not by the core.
 */
export interface ComponentDefinition {
  type: string
  label: string
  icon?: ReactNode
  /** Which component types can be children of this component */
  allowedChildren?: string[]
  /** Whether this component can contain text content */
  hasContent?: boolean
  /** Whether this component is a container (can have children) */
  isContainer?: boolean
  /** Default props when a new instance is created */
  defaultProps?: Record<string, unknown>
  /** Default styles when a new instance is created */
  defaultStyles?: Record<string, unknown>
}

/**
 * The adapter interface that a renderer (e.g. MJML) must implement.
 * The core builder delegates rendering and component definition to the adapter.
 */
export interface BuilderAdapter {
  /** All available components the user can drag into the builder */
  components: ComponentDefinition[]

  /** Render the configuration panel for the selected node */
  renderConfigPanel(
    node: BuilderNode,
    onUpdate: (updates: Partial<Pick<BuilderNode, 'props' | 'styles' | 'content'>>) => void,
  ): ReactNode

  /** Render a preview of the full tree (e.g. as HTML string for an iframe) */
  renderPreview(tree: BuilderNode[]): string

  /** Render a visual block for the canvas (optional — falls back to label if not provided) */
  renderVisualBlock?(
    node: BuilderNode,
    isSelected: boolean,
    children: ReactNode,
  ): ReactNode

  /** Get the default props/styles for a new node of the given type */
  getDefaultNode(type: string): Omit<BuilderNode, 'id'>
}

/**
 * The builder state exposed to consumers via context/hooks.
 */
export interface BuilderState {
  tree: BuilderNode[]
  selectedNodeId: string | null
}

export interface BuilderActions {
  /** Add a node as a child of parentId, or at root if parentId is null */
  addNode(type: string, parentId: string | null, index?: number): void
  /** Remove a node by id (and all its children) */
  removeNode(nodeId: string): void
  /** Move a node to a new parent/position */
  moveNode(nodeId: string, newParentId: string | null, newIndex: number): void
  /** Update a node's props, styles, or content */
  updateNode(nodeId: string, updates: Partial<Pick<BuilderNode, 'props' | 'styles' | 'content'>>): void
  /** Select a node (or null to deselect) */
  selectNode(nodeId: string | null): void
  /** Replace the entire tree */
  setTree(tree: BuilderNode[]): void
}
