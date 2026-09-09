import type { ReactNode } from 'react'

interface ControlRowProps {
  label: string
  /**
   * When set and `!== value`, a small dot is shown next to the label —
   * clicking it resets the field to `defaultValue` via `onReset`. Mirrors
   * Webflow's "modified" affordance, where every customised property gets a
   * coloured marker and one-click revert.
   */
  modified?: boolean
  onReset?: () => void
  children: ReactNode
}

/**
 * Webflow-style dense row: label takes the left ~40%, control fills the
 * right side. Used by every theme-token control (ColorPicker, NumberInput)
 * so the whole panel reads as a consistent table-of-properties — eyeball
 * scanning down the labels is faster than scanning stacked label-above-
 * input blocks.
 */
export function ControlRow({ label, modified, onReset, children }: ControlRowProps) {
  return (
    <div className='flex min-h-7 items-center gap-2'>
      <div className='flex w-[42%] shrink-0 items-center gap-1.5'>
        {modified ? (
          <button
            type='button'
            onClick={onReset}
            className='h-1.5 w-1.5 shrink-0 rounded-full bg-orange-500 transition-transform hover:scale-125'
            title='Reset to default'
            aria-label={`Reset ${label} to default`}
          />
        ) : (
          // Reserve the same hit-zone so the label column stays aligned
          // whether or not the field is modified — keeps the panel visually
          // stable while editing.
          <span className='h-1.5 w-1.5 shrink-0' aria-hidden='true' />
        )}
        <span className='truncate text-xs text-muted-foreground'>{label}</span>
      </div>
      <div className='flex min-w-0 flex-1 items-center'>{children}</div>
    </div>
  )
}
