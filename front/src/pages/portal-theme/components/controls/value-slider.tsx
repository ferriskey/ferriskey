import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Slider } from '@/components/ui/slider'

type ValueSliderProps = {
  label: string
  value: number
  onChange: (value: number) => void
  min: number
  max: number
  step?: number
  unit?: string
}

export function ValueSlider({
  label,
  value,
  onChange,
  min,
  max,
  step = 1,
  unit,
}: ValueSliderProps) {
  return (
    <div className='flex flex-col gap-1.5'>
      <div className='flex items-center justify-between'>
        <Label className='text-xs font-medium text-muted-foreground'>{label}</Label>
        <div className='flex items-center gap-1'>
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
            className='h-7 w-16 text-right text-xs'
          />
          {unit && <span className='text-xs text-muted-foreground'>{unit}</span>}
        </div>
      </div>
      <Slider
        value={[value]}
        min={min}
        max={max}
        step={step}
        onValueChange={(values) => onChange(values[0])}
      />
    </div>
  )
}
