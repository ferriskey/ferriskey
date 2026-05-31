import { HexColorPicker } from 'react-colorful'
import type { ReactNode } from 'react'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'

interface FieldProps {
  label: string
  value: string | undefined
  onChange: (value: string) => void
  placeholder?: string
}

/**
 * Webflow-style dense row used by every block config field: label left,
 * control right. Pulled out so all three field components share one shape
 * (TextField / ColorField / SelectField) — admins scan a column of labels
 * instead of zig-zagging through stacked label-above-input blocks.
 */
function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className='flex min-h-7 items-center gap-2'>
      <span className='w-[42%] shrink-0 truncate text-xs text-muted-foreground'>
        {label}
      </span>
      <div className='flex min-w-0 flex-1 items-center'>{children}</div>
    </div>
  )
}

export function TextField({ label, value, onChange, placeholder }: FieldProps) {
  return (
    <Row label={label}>
      <input
        type='text'
        className='h-7 w-full rounded border border-border bg-background px-2 text-[11px]'
        value={value ?? ''}
        placeholder={placeholder}
        onChange={(e) => onChange(e.target.value)}
      />
    </Row>
  )
}

/**
 * Color picker mirrors the theme panel's ColorPicker: small swatch on the
 * left (acts as a popover trigger for `react-colorful`'s hex picker), then a
 * hex input filling the row. Hides the native `<input type="color">` because
 * its picker UI varies wildly across OSes and doesn't accept rgba/var()
 * tokens. Empty string is rendered as a transparent swatch.
 */
export function ColorField({ label, value, onChange }: FieldProps) {
  const swatchValue = value && value.length > 0 ? value : 'transparent'
  return (
    <Row label={label}>
      <div className='flex w-full items-center gap-1.5'>
        <Popover>
          <PopoverTrigger asChild>
            <button
              type='button'
              className='h-6 w-6 shrink-0 rounded border border-border'
              style={{ backgroundColor: swatchValue }}
              aria-label={`Pick color for ${label}`}
            />
          </PopoverTrigger>
          <PopoverContent className='w-auto p-2' align='start'>
            <HexColorPicker
              color={value && value.startsWith('#') ? value : '#000000'}
              onChange={onChange}
            />
          </PopoverContent>
        </Popover>
        <input
          type='text'
          className='h-7 flex-1 rounded border border-border bg-background px-2 font-mono text-[10px] uppercase'
          value={value ?? ''}
          onChange={(e) => onChange(e.target.value)}
        />
      </div>
    </Row>
  )
}

interface SelectFieldProps extends Omit<FieldProps, 'onChange' | 'placeholder'> {
  options: { label: string; value: string }[]
  onChange: (value: string) => void
  allowEmpty?: boolean
}

export function SelectField({ label, value, options, onChange, allowEmpty = true }: SelectFieldProps) {
  return (
    <Row label={label}>
      <select
        className='h-7 w-full rounded border border-border bg-background px-1.5 text-[11px]'
        value={value ?? ''}
        onChange={(e) => onChange(e.target.value)}
      >
        {allowEmpty && <option value=''>—</option>}
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </Row>
  )
}
