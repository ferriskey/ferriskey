import { HexColorPicker } from 'react-colorful'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Input } from '@/components/ui/input'
import { ControlRow } from './control-row'

type ColorPickerProps = {
  label: string
  value: string
  onChange: (value: string) => void
  /**
   * When set, a "modified" dot appears next to the label and clicking it
   * snaps the value back to `defaultValue`. Pass it from each panel using
   * the corresponding entry in `defaultTheme.colors`.
   */
  defaultValue?: string
}

/**
 * Compact color row à la Webflow: tiny swatch (acts as popover trigger),
 * then a hex input filling the remaining space. The whole control fits on a
 * single line so a panel can stack 6–10 colors at a glance.
 */
export function ColorPicker({ label, value, onChange, defaultValue }: ColorPickerProps) {
  const modified = defaultValue !== undefined && defaultValue !== value
  return (
    <ControlRow
      label={label}
      modified={modified}
      onReset={defaultValue !== undefined ? () => onChange(defaultValue) : undefined}
    >
      <div className='flex w-full items-center gap-1.5'>
        <Popover>
          <PopoverTrigger asChild>
            <button
              type='button'
              className='h-6 w-6 shrink-0 rounded border border-border'
              style={{ backgroundColor: value }}
              aria-label={`Pick color for ${label}`}
            />
          </PopoverTrigger>
          <PopoverContent className='w-auto p-2' align='start'>
            <HexColorPicker color={value} onChange={onChange} />
          </PopoverContent>
        </Popover>
        <Input
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className='h-7 flex-1 font-mono text-[11px] uppercase'
        />
      </div>
    </ControlRow>
  )
}
