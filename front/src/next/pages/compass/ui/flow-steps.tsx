import { Check, Loader, Minus, X } from 'lucide-react'
import { Pill } from '@/components/kit'
import { cn } from '@/lib/utils'
import { Schemas } from '@/api/api.client'
import {
  formatDuration,
  formatTime,
  stepDescriptions,
  stepLabel,
  stepStatusTone,
} from '../flow-format'

import CompassFlowStep = Schemas.CompassFlowStep
import StepStatus = Schemas.StepStatus

function StepIcon({ status }: { status: StepStatus }) {
  if (status === 'success') return <Check className='size-3' strokeWidth={3} />
  if (status === 'failure') return <X className='size-3' strokeWidth={3} />
  return <Minus className='size-3' strokeWidth={3} />
}

interface StepRowProps {
  step: CompassFlowStep
  index: number
  last: boolean
  maxMs: number
}

function StepRow({ step, index, last, maxMs }: StepRowProps) {
  const share = step.duration_ms ? (step.duration_ms / maxMs) * 100 : 0

  return (
    <li className='flex gap-3'>
      <div className='flex w-5 shrink-0 flex-col items-center'>
        <span
          className={cn(
            'grid size-5 shrink-0 place-items-center rounded-full',
            step.status === 'success' && 'bg-fk-success-soft text-fk-success',
            step.status === 'failure' && 'bg-fk-danger-soft text-fk-danger',
            step.status === 'skipped' && 'bg-neutral-100 text-neutral-400 dark:bg-neutral-800 dark:text-neutral-500'
          )}
        >
          <StepIcon status={step.status} />
        </span>
        {!last && <span className='w-px flex-1 bg-fk-line-soft' />}
      </div>

      <div className={cn('min-w-0 flex-1', last ? 'pb-1' : 'pb-4')}>
        <div className='flex flex-wrap items-center gap-2'>
          <span className='tnum text-[11px] text-neutral-300 dark:text-neutral-600'>{index + 1}</span>
          <p className='text-xs font-medium text-neutral-900 dark:text-neutral-100'>{stepLabel(step)}</p>
          <Pill tone={stepStatusTone[step.status]} mono>
            {step.status}
          </Pill>
          <span className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
            {step.step_name}
          </span>
          <span className='tnum text-[11px] text-neutral-400 dark:text-neutral-500'>
            {formatTime(step.started_at)}
          </span>
        </div>

        <p className='mt-0.5 text-xs text-neutral-500 dark:text-neutral-400'>
          {step.status === 'skipped'
            ? 'Not applicable to this execution.'
            : stepDescriptions[step.step_name]}
        </p>

        {(step.error_code || step.error_message) && (
          <p className='mt-1 text-xs text-fk-danger'>
            {step.error_code && <span className='font-mono-ui'>{step.error_code}</span>}
            {step.error_code && step.error_message ? ' — ' : ''}
            {step.error_message}
          </p>
        )}

        {step.duration_ms != null && (
          <div className='mt-1.5 flex items-center gap-2'>
            <div className='h-1 min-w-0 flex-1 overflow-hidden rounded-full bg-neutral-100 dark:bg-neutral-800'>
              <div
                className={cn(
                  'h-full rounded-full',
                  step.status === 'failure' ? 'bg-fk-danger' : 'bg-fk-info'
                )}
                style={{ width: `${Math.max(share, 2)}%` }}
              />
            </div>
            <span className='tnum shrink-0 text-[11px] text-neutral-500 dark:text-neutral-400'>
              {formatDuration(step.duration_ms)}
            </span>
          </div>
        )}
      </div>
    </li>
  )
}

export interface FlowStepsProps {
  steps: CompassFlowStep[]
  pending: boolean
}

export default function FlowSteps({ steps, pending }: FlowStepsProps) {
  const maxMs = Math.max(...steps.map((s) => s.duration_ms ?? 0), 1)

  return (
    <ol className='py-2'>
      {steps.map((step, index) => (
        <StepRow
          key={step.id}
          step={step}
          index={index}
          last={index === steps.length - 1 && !pending}
          maxMs={maxMs}
        />
      ))}
      {pending && (
        <li className='flex gap-3'>
          <div className='flex w-5 shrink-0 justify-center'>
            <span className='grid size-5 place-items-center rounded-full border border-dashed border-fk-line text-neutral-300 dark:text-neutral-600'>
              <Loader className='size-3 animate-spin' strokeWidth={2.5} />
            </span>
          </div>
          <p className='pb-2 pt-0.5 text-xs text-neutral-400 dark:text-neutral-500'>
            Waiting for the next step…
          </p>
        </li>
      )}
    </ol>
  )
}
