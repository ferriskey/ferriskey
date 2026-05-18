import { Trash2 } from 'lucide-react'
import { useBuilderContext } from '../context'
import { useSelectedNode } from '../hooks'
import { useEditingBreakpoint } from '../breakpoint-context'
import { BREAKPOINTS, type Breakpoint, type BuilderNode } from '../types'

export function ConfigPanel() {
  const { adapter, updateNode, removeNode, selectedNodeId } = useBuilderContext()
  const selectedNode = useSelectedNode()
  const { current: editingBp, setCurrent: setEditingBp } = useEditingBreakpoint()

  if (!selectedNode) {
    return (
      <div className='flex h-full items-center justify-center p-4 text-sm text-muted-foreground'>
        Select a component to configure
      </div>
    )
  }

  const componentDef = adapter.components.find((c) => c.type === selectedNode.type)

  // When editing a non-base breakpoint, we present the adapter with a node
  // whose `props` are already merged through the *full* mobile-first cascade
  // (base ← sm ← md ← ... up to and including the editing bp) so the form
  // shows what the user would actually see at that viewport. Without the full
  // cascade, the form would display `base` values that a smaller-bp override
  // is silently masking — leading users to think they've set md=repeat(2)
  // when in fact sm=repeat(1) is still winning at md width.
  // Edits still get routed to the editing-bp override layer only.
  const effectiveNode: BuilderNode = editingBp
    ? (() => {
        let props: Record<string, unknown> = { ...selectedNode.props }
        for (const bp of BREAKPOINTS) {
          const override = selectedNode.breakpoints?.[bp]
          if (override) props = { ...props, ...override }
          if (bp === editingBp) break
        }
        return { ...selectedNode, props }
      })()
    : selectedNode

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

      <BreakpointTabs
        node={selectedNode}
        current={editingBp}
        onChange={setEditingBp}
      />

      <div className='flex flex-col gap-0 p-3'>
        {adapter.renderConfigPanel(effectiveNode, (updates) => {
          if (!selectedNodeId) return
          if (editingBp) {
            // Route prop / content / styles edits to the bp override layer.
            // Name stays on the base node — same identifier across breakpoints.
            const { name, props, styles, content, breakpoints } = updates
            const routed: Parameters<typeof updateNode>[1] = {}
            if (name !== undefined) routed.name = name
            if (breakpoints) routed.breakpoints = breakpoints
            if (props || styles || content !== undefined) {
              routed.breakpoints = {
                ...(routed.breakpoints ?? {}),
                [editingBp]: {
                  ...(props ?? {}),
                  // Styles & content overrides aren't supported by the CSS
                  // generator yet; we still store them so adapters can use
                  // them, but the resolver mostly cares about `props`.
                  ...(styles ?? {}),
                  ...(content !== undefined ? { content } : {}),
                },
              }
            }
            updateNode(selectedNodeId, routed)
          } else {
            updateNode(selectedNodeId, updates)
          }
        })}
      </div>
    </div>
  )
}

function BreakpointTabs({
  node,
  current,
  onChange,
}: {
  node: BuilderNode
  current: Breakpoint | null
  onChange: (bp: Breakpoint | null) => void
}) {
  const baseActive = current === null
  return (
    <div className='flex flex-col gap-1 border-b border-border bg-muted/30 px-2 py-1.5'>
      <div className='flex items-center gap-1'>
        <BpTab
          active={baseActive}
          onClick={() => onChange(null)}
          // The base layer is the mobile-first foundation: it applies at
          // every width and is what the design degrades to on the smallest
          // screen. Bigger breakpoints only progressively enhance on top.
          label='Base'
          hasOverrides={false}
        />
        {BREAKPOINTS.map((bp) => (
          <BpTab
            key={bp}
            active={current === bp}
            onClick={() => onChange(bp)}
            label={bp}
            hasOverrides={Boolean(node.breakpoints?.[bp])}
          />
        ))}
      </div>
      <p className='px-0.5 text-[10px] leading-tight text-muted-foreground'>
        {baseActive
          ? 'Mobile-first: these values apply at every width unless a larger breakpoint overrides them.'
          : `Overrides apply from min-width: ${BREAKPOINT_WIDTH_LABEL[current]} and up.`}
      </p>
    </div>
  )
}

const BREAKPOINT_WIDTH_LABEL: Record<Breakpoint, string> = {
  sm: '640px',
  md: '768px',
  lg: '1024px',
  xl: '1280px',
}

function BpTab({
  active,
  onClick,
  label,
  hasOverrides,
}: {
  active: boolean
  onClick: () => void
  label: string
  hasOverrides: boolean
}) {
  return (
    <button
      type='button'
      onClick={onClick}
      className={`relative rounded px-2 py-0.5 text-[11px] font-medium uppercase tracking-wider transition-colors ${
        active
          ? 'bg-background text-foreground shadow-sm'
          : 'text-muted-foreground hover:text-foreground'
      }`}
    >
      {label}
      {hasOverrides && (
        <span
          className='absolute -right-0.5 -top-0.5 h-1.5 w-1.5 rounded-full bg-primary'
          aria-hidden
        />
      )}
    </button>
  )
}
