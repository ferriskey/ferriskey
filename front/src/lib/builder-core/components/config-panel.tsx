import { Trash2 } from 'lucide-react'
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

  const componentDef = adapter.components.find(
    (c) => c.type === selectedNode.type,
  )

  return (
    <div className='flex flex-col'>
      <div className='flex items-center gap-2 border-b border-border px-3 py-2.5'>
        {componentDef?.icon && (
          <span className='flex h-5 w-5 shrink-0 items-center justify-center text-muted-foreground'>
            {componentDef.icon}
          </span>
        )}
        <h3 className='flex-1 text-sm font-medium'>
          {componentDef?.label ?? selectedNode.type}
        </h3>
        <button
          type='button'
          className='flex h-7 w-7 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive'
          onClick={() => removeNode(selectedNode.id)}
        >
          <Trash2 className='h-3.5 w-3.5' />
        </button>
      </div>

      <div className='flex flex-col gap-0 p-3'>
        {adapter.renderConfigPanel(selectedNode, (updates) => {
          if (selectedNodeId) {
            updateNode(selectedNodeId, updates)
          }
        })}
      </div>
    </div>
  )
}
