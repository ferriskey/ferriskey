import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { usePortalThemeContext } from '../../context/portal-theme-context'
import type { ThemeFontStyle, ThemeFontLinkStyle, ThemeFonts } from '../../lib/theme'
import { ValueSlider } from '../controls/value-slider'

type StyleKey = Exclude<keyof ThemeFonts, 'url' | 'baseSize' | 'links'>

const STYLE_FIELDS: Array<{ key: StyleKey; label: string }> = [
  { key: 'title', label: 'Title' },
  { key: 'subtitle', label: 'Subtitle' },
  { key: 'body', label: 'Body' },
  { key: 'buttons', label: 'Buttons' },
  { key: 'inputLabels', label: 'Input labels' },
]

function StyleControls({
  label,
  value,
  onChange,
}: {
  label: string
  value: ThemeFontStyle
  onChange: (next: ThemeFontStyle) => void
}) {
  return (
    <div className='flex flex-col gap-2 rounded-md border border-border p-3'>
      <div className='text-xs font-semibold'>{label}</div>
      <ValueSlider
        label='Weight'
        value={value.weight}
        onChange={(weight) => onChange({ ...value, weight })}
        min={100}
        max={900}
        step={100}
      />
      <ValueSlider
        label='Size'
        value={value.sizePct}
        onChange={(sizePct) => onChange({ ...value, sizePct })}
        min={50}
        max={200}
        step={2.5}
        unit='%'
      />
    </div>
  )
}

export function FontsPanel() {
  const { theme, setFont } = usePortalThemeContext()
  const { fonts } = theme

  const handleLinkChange = (next: ThemeFontLinkStyle) => setFont('links', next)

  return (
    <div className='flex flex-col gap-4'>
      <div>
        <h2 className='text-sm font-semibold'>Fonts</h2>
        <p className='text-xs text-muted-foreground'>Typography tokens used across the portal.</p>
      </div>

      <div className='flex flex-col gap-1.5'>
        <Label className='text-xs font-medium text-muted-foreground'>Custom font URL</Label>
        <Input
          value={fonts.url ?? ''}
          placeholder='https://fonts.googleapis.com/css2?family=Inter'
          onChange={(e) => setFont('url', e.target.value || null)}
          className='h-8 text-xs'
        />
      </div>

      <ValueSlider
        label='Base size'
        value={fonts.baseSize}
        onChange={(baseSize) => setFont('baseSize', baseSize)}
        min={12}
        max={20}
        unit='px'
      />

      {STYLE_FIELDS.map(({ key, label }) => (
        <StyleControls
          key={key}
          label={label}
          value={fonts[key]}
          onChange={(next) => setFont(key, next)}
        />
      ))}

      <div className='flex flex-col gap-2 rounded-md border border-border p-3'>
        <div className='text-xs font-semibold'>Links</div>
        <ValueSlider
          label='Weight'
          value={fonts.links.weight}
          onChange={(weight) => handleLinkChange({ ...fonts.links, weight })}
          min={100}
          max={900}
          step={100}
        />
        <ValueSlider
          label='Size'
          value={fonts.links.sizePct}
          onChange={(sizePct) => handleLinkChange({ ...fonts.links, sizePct })}
          min={50}
          max={200}
          step={2.5}
          unit='%'
        />
        <div className='flex flex-col gap-1.5'>
          <Label className='text-xs font-medium text-muted-foreground'>Decoration</Label>
          <Select
            value={fonts.links.style}
            onValueChange={(style: ThemeFontLinkStyle['style']) =>
              handleLinkChange({ ...fonts.links, style })
            }
          >
            <SelectTrigger className='h-8 w-full text-xs'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value='normal'>Normal</SelectItem>
              <SelectItem value='underline'>Underline</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
  )
}
