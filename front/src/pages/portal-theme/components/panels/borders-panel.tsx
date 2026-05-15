import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { usePortalThemeContext } from '../../context/portal-theme-context'
import type { ThemeBorders, ThemeShadow } from '../../lib/theme'
import { ValueSlider } from '../controls/value-slider'

type SliderField = {
  key: Exclude<keyof ThemeBorders, 'widgetShadow'>
  label: string
  min: number
  max: number
  unit: string
}

const RADIUS_FIELDS: SliderField[] = [
  { key: 'buttonRadius', label: 'Button radius', min: 0, max: 32, unit: 'px' },
  { key: 'buttonBorderWeight', label: 'Button border', min: 0, max: 6, unit: 'px' },
  { key: 'inputRadius', label: 'Input radius', min: 0, max: 32, unit: 'px' },
  { key: 'inputBorderWeight', label: 'Input border', min: 0, max: 6, unit: 'px' },
  { key: 'widgetRadius', label: 'Widget radius', min: 0, max: 32, unit: 'px' },
  { key: 'widgetBorderWeight', label: 'Widget border', min: 0, max: 6, unit: 'px' },
]

export function BordersPanel() {
  const { theme, setBorder } = usePortalThemeContext()
  const { borders } = theme

  return (
    <div className='flex flex-col gap-4'>
      <div>
        <h2 className='text-sm font-semibold'>Borders & widget</h2>
        <p className='text-xs text-muted-foreground'>Radius, border thickness and widget shadow.</p>
      </div>

      <div className='flex flex-col gap-3'>
        {RADIUS_FIELDS.map(({ key, label, min, max, unit }) => (
          <ValueSlider
            key={key}
            label={label}
            value={borders[key]}
            onChange={(next) => setBorder(key, next)}
            min={min}
            max={max}
            unit={unit}
          />
        ))}

        <div className='flex flex-col gap-1.5'>
          <Label className='text-xs font-medium text-muted-foreground'>Widget shadow</Label>
          <Select
            value={borders.widgetShadow}
            onValueChange={(value: ThemeShadow) => setBorder('widgetShadow', value)}
          >
            <SelectTrigger className='h-8 w-full text-xs'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value='none'>None</SelectItem>
              <SelectItem value='small'>Small</SelectItem>
              <SelectItem value='large'>Large</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
  )
}
