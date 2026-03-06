import { Schemas } from '@/api/api.client'
import { formatSnakeCaseToTitleCase } from '@/utils'
import { CheckCircle, Clock, XCircle, MinusCircle } from 'lucide-react'
import { useMemo, useCallback } from 'react'
import {
  ReactFlow,
  Background,
  BackgroundVariant,
  type Node,
  type Edge,
  Position,
  Handle,
  type NodeProps,
  useNodesState,
  useEdgesState,
  MarkerType,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import CompassFlowStep = Schemas.CompassFlowStep
import StepStatus = Schemas.StepStatus

interface FlowTimelineProps {
  steps: CompassFlowStep[]
}

const statusConfig: Record<StepStatus, { bg: string; border: string; text: string; edge: string; iconColor: string }> = {
  success: { bg: '#f0fdf4', border: '#86efac', text: '#16a34a', edge: '#86efac', iconColor: 'text-emerald-500' },
  failure: { bg: '#fef2f2', border: '#fca5a5', text: '#dc2626', edge: '#fca5a5', iconColor: 'text-red-500' },
  skipped: { bg: '#f9fafb', border: '#d1d5db', text: '#6b7280', edge: '#d1d5db', iconColor: 'text-gray-400' },
}

const formatTime = (timestamp: string) => {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return '--:--:--'
  return new Intl.DateTimeFormat(undefined, { timeStyle: 'medium' }).format(date)
}

type StepNodeData = {
  step: CompassFlowStep
  hasTop: boolean
  hasBottom: boolean
  hasLeft: boolean
  hasRight: boolean
}

function StepNode({ data }: NodeProps<Node<StepNodeData>>) {
  const { step, hasTop, hasBottom, hasLeft, hasRight } = data
  const config = statusConfig[step.status]

  const icon = step.status === 'success'
    ? <CheckCircle className={`h-4 w-4 ${config.iconColor}`} />
    : step.status === 'failure'
      ? <XCircle className={`h-4 w-4 ${config.iconColor}`} />
      : <MinusCircle className={`h-4 w-4 ${config.iconColor}`} />

  const handleClass = '!bg-transparent !border-0 !w-0 !h-0'

  return (
    <div
      className='rounded-lg p-4 w-[220px]'
      style={{
        backgroundColor: config.bg,
        border: `2px dashed ${config.border}`,
      }}
    >
      {hasTop && <Handle id='top' type='target' position={Position.Top} className={handleClass} />}
      {hasBottom && <Handle id='bottom' type='source' position={Position.Bottom} className={handleClass} />}
      {hasLeft && <Handle id='left' type='target' position={Position.Left} className={handleClass} />}
      {hasRight && <Handle id='right' type='source' position={Position.Right} className={handleClass} />}

      <div className='flex items-center gap-2 mb-2'>
        {icon}
        <span className='text-sm font-semibold text-foreground truncate'>
          {formatSnakeCaseToTitleCase(step.step_name)}
        </span>
      </div>

      <div className='flex items-center gap-2 mb-1.5'>
        <span
          className='inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-bold uppercase tracking-wide'
          style={{ color: config.text, backgroundColor: `${config.border}33` }}
        >
          {step.status}
        </span>
        {step.duration_ms != null && (
          <span className='text-[11px] font-mono text-muted-foreground'>{step.duration_ms}ms</span>
        )}
      </div>

      <div className='flex items-center gap-1 text-[11px] text-muted-foreground'>
        <Clock className='h-3 w-3' />
        {formatTime(step.started_at)}
      </div>

      {step.error_code && (
        <div className='mt-2 pt-2 border-t border-border/50 text-[11px] text-red-500 font-mono truncate'>
          {step.error_code}
        </div>
      )}
    </div>
  )
}

const nodeTypes = { step: StepNode }

const NODE_WIDTH = 220
const NODE_HEIGHT = 120
const ROW_GAP = 40
const COL_GAP = 100

interface LayoutInfo {
  col: number
  row: number
}

function buildGraph(steps: CompassFlowStep[]): { nodes: Node<StepNodeData>[]; edges: Edge[]; maxRows: number } {
  const sorted = [...steps].sort(
    (a, b) => new Date(a.started_at).getTime() - new Date(b.started_at).getTime(),
  )

  const nodes: Node<StepNodeData>[] = []
  const edges: Edge[] = []
  const layouts: LayoutInfo[] = []

  // Compute layout positions
  let col = 0
  let row = 0
  let prevStepName: string | null = null
  let maxRows = 1

  for (let i = 0; i < sorted.length; i++) {
    const step = sorted[i]
    const sameAsPrev = prevStepName === step.step_name

    if (sameAsPrev) {
      row++
      maxRows = Math.max(maxRows, row + 1)
    } else if (i > 0) {
      col++
      row = 0
    }

    layouts.push({ col, row })
    prevStepName = step.step_name
  }

  // Build nodes with correct handles based on edge directions
  for (let i = 0; i < sorted.length; i++) {
    const step = sorted[i]
    const layout = layouts[i]
    const x = layout.col * (NODE_WIDTH + COL_GAP)
    const y = layout.row * (NODE_HEIGHT + ROW_GAP)

    // Determine which handles this node needs
    let hasTop = false
    let hasBottom = false
    let hasLeft = false
    let hasRight = false

    // Check incoming edge (from previous node)
    if (i > 0) {
      const prevLayout = layouts[i - 1]
      if (prevLayout.col === layout.col) {
        hasTop = true // same column → comes from above
      } else {
        hasLeft = true // different column → comes from left
      }
    }

    // Check outgoing edge (to next node)
    if (i < sorted.length - 1) {
      const nextLayout = layouts[i + 1]
      if (nextLayout.col === layout.col) {
        hasBottom = true // same column → goes down
      } else {
        hasRight = true // different column → goes right
      }
    }

    nodes.push({
      id: step.id,
      type: 'step',
      position: { x, y },
      data: { step, hasTop, hasBottom, hasLeft, hasRight },
      draggable: false,
    })
  }

  // Build edges
  for (let i = 0; i < sorted.length - 1; i++) {
    const sourceStep = sorted[i]
    const targetStep = sorted[i + 1]
    const sourceLayout = layouts[i]
    const targetLayout = layouts[i + 1]
    const edgeColor = statusConfig[sourceStep.status].edge

    const sameCol = sourceLayout.col === targetLayout.col
    const sourceHandle = sameCol ? 'bottom' : 'right'
    const targetHandle = sameCol ? 'top' : 'left'

    edges.push({
      id: `e-${sourceStep.id}-${targetStep.id}`,
      source: sourceStep.id,
      target: targetStep.id,
      sourceHandle,
      targetHandle,
      style: {
        stroke: edgeColor,
        strokeWidth: 2,
        strokeDasharray: '6 4',
      },
      markerEnd: {
        type: MarkerType.ArrowClosed,
        color: edgeColor,
        width: 16,
        height: 16,
      },
    })
  }

  return { nodes, edges, maxRows }
}

export function FlowTimeline({ steps }: FlowTimelineProps) {
  const { nodes: initialNodes, edges: initialEdges, maxRows } = useMemo(() => buildGraph(steps), [steps])

  const [nodes] = useNodesState(initialNodes)
  const [edges] = useEdgesState(initialEdges)

  const onInit = useCallback((instance: { fitView: () => void }) => {
    setTimeout(() => instance.fitView(), 0)
  }, [])

  if (steps.length === 0) {
    return (
      <div className='flex flex-col items-center justify-center py-16 text-muted-foreground'>
        <Clock className='h-8 w-8 mb-3 opacity-40' />
        <p>No steps recorded for this flow.</p>
      </div>
    )
  }

  const graphHeight = Math.max(280, maxRows * (NODE_HEIGHT + ROW_GAP) + 80)

  return (
    <div className='w-full' style={{ height: `${graphHeight}px` }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onInit={onInit}
        fitView
        fitViewOptions={{ padding: 0.05, maxZoom: 0.45 }}
        proOptions={{ hideAttribution: true }}
        panOnDrag
        zoomOnScroll={false}
        zoomOnPinch
        zoomOnDoubleClick={false}
        nodesDraggable={false}
        nodesConnectable={false}
        elementsSelectable={false}
        minZoom={0.2}
        maxZoom={1.5}
      >
        <Background variant={BackgroundVariant.Dots} gap={20} size={1} />
      </ReactFlow>
    </div>
  )
}
