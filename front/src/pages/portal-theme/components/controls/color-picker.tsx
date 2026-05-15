import { HexColorPicker } from 'react-colorful'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

type ColorPickerProps = {
  label: string
  value: string
  onChange: (value: string) => void
}

export function ColorPicker({ label, value, onChange }: ColorPickerProps) {
  return (
    <div className='flex flex-col gap-1.5'>
      <Label className='text-xs font-medium text-muted-foreground'>{label}</Label>
      <div className='flex items-center gap-2'>
        <Popover>
          <PopoverTrigger asChild>
            <button
              type='button'
              className='h-8 w-8 shrink-0 rounded-md border border-border'
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
          className='h-8 flex-1 font-mono text-xs uppercase'
        />
      </div>
    </div>
  )
}
