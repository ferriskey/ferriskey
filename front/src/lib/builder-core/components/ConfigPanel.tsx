import { useBuilderContext } from '../context'
import { useSelectedNode } from '../hooks'

export function ConfigPanel() {
  const { adapter, updateNode, removeNode, selectedNodeId } =
    useBuilderContext()
  const selectedNode = useSelectedNode()

  if (!selectedNode) {
    return (
      <div className='flex h-full items-center justify-center p-4 text-sm text-muted-foreground'>
        Select a component to configure
      </div>
    )
  }

  return (
    <div className='flex flex-col gap-3 p-3'>
      <div className='flex items-center justify-between'>
        <h3 className='text-sm font-medium'>
          {adapter.components.find((c) => c.type === selectedNode.type)
            ?.label ?? selectedNode.type}
        </h3>
        <button
          type='button'
          className='rounded px-2 py-1 text-xs text-destructive hover:bg-destructive/10'
          onClick={() => removeNode(selectedNode.id)}
        >
          Delete
        </button>
      </div>

      <div className='h-px bg-border' />

      {adapter.renderConfigPanel(selectedNode, (updates) => {
        if (selectedNodeId) {
          updateNode(selectedNodeId, updates)
        }
      })}
    </div>
  )
}
