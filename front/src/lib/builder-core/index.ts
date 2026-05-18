// Types
export type {
  BuilderNode,
  BuilderAdapter,
  BuilderActions,
  BuilderState,
  ComponentDefinition,
} from './types'

// Context & Provider
export { BuilderProvider, useBuilderContext } from './context'

// Hooks
export { useBuilder, useSelectedNode, useComponentLibrary } from './hooks'

// Components
export { Canvas } from './components/canvas'
export { ComponentLibrary } from './components/component-library'
export { ConfigPanel } from './components/config-panel'
export { BuilderDragOverlay } from './components/drag-overlay'
export { BuilderShell } from './components/builder-shell'
export { PresetLibrary } from './components/preset-library'

// Utils
export { findNode, findNodePath, generateNodeId, regenerateIds } from './utils'

// Breadcrumb (selection navigation)
export { SelectionBreadcrumb } from './components/selection-breadcrumb'
