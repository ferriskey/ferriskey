import { Input } from '@/components/ui/input'
import { ControlRow } from './control-row'

type ValueSliderProps = {
  label: string
  value: number
  onChange: (value: number) => void
  min: number
  max: number
  step?: number
  unit?: string
  /**
   * When set, a "modified" dot appears next to the label and clicking it
   * snaps the value back to `defaultValue`. Pass it from each panel using
   * the corresponding entry in `defaultTheme`.
   */
  defaultValue?: number
}

/**
 * Compact numeric input à la Webflow: label on the left, native
 * `<input type="number">` (with built-in steppers) on the right plus an
 * optional unit suffix.
 *
 * The component is still named `ValueSlider` for the moment to avoid a
 * sweeping rename across every panel — under the hood it's now a numeric
 * input. We dropped the actual slider because token values (border weight,
 * font size, radius) benefit far more from precise stepping than from a
 * fuzzy drag handle; the native input's keyboard support (↑/↓, ⇧↑/⇧↓ for
 * larger steps in some browsers) covers exploration cases anyway.
 */
export function ValueSlider({
  label,
  value,
  onChange,
  min,
  max,
  step = 1,
  unit,
  defaultValue,
}: ValueSliderProps) {
  const modified = defaultValue !== undefined && defaultValue !== value
  return (
    <ControlRow
      label={label}
      modified={modified}
      onReset={defaultValue !== undefined ? () => onChange(defaultValue) : undefined}
    >
      <div className='flex w-full items-center gap-1'>
        <Input
          type='number'
          value={value}
          min={min}
          max={max}
          step={step}
          onChange={(e) => {
            const next = Number(e.target.value)
            if (!Number.isNaN(next)) onChange(next)
          }}
          className='h-7 w-full text-right text-xs tabular-nums'
        />
        {unit && (
          <span className='w-4 shrink-0 text-[10px] text-muted-foreground'>{unit}</span>
        )}
      </div>
    </ControlRow>
  )
}
