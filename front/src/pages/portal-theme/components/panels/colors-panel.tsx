import { usePortalThemeContext } from '../../context/portal-theme-context'
import type { ThemeColors } from '../../lib/theme'
import { ColorPicker } from '../controls/color-picker'

const COLOR_FIELDS: Array<{ key: keyof ThemeColors; label: string }> = [
  { key: 'primaryButton', label: 'Primary button' },
  { key: 'primaryButtonLabel', label: 'Primary button label' },
  { key: 'secondaryButton', label: 'Secondary button' },
  { key: 'secondaryButtonLabel', label: 'Secondary button label' },
  { key: 'widgetBackground', label: 'Widget background' },
  { key: 'pageBackground', label: 'Page background' },
  { key: 'bodyText', label: 'Body text' },
  { key: 'links', label: 'Links' },
  { key: 'error', label: 'Error' },
]

export function ColorsPanel() {
  const { theme, setColor } = usePortalThemeContext()

  return (
    <div className='flex flex-col gap-4'>
      <div>
        <h2 className='text-sm font-semibold'>Colors</h2>
        <p className='text-xs text-muted-foreground'>
          Define the palette used across all portal pages.
        </p>
      </div>
      <div className='flex flex-col gap-3'>
        {COLOR_FIELDS.map(({ key, label }) => (
          <ColorPicker
            key={key}
            label={label}
            value={theme.colors[key]}
            onChange={(value) => setColor(key, value)}
          />
        ))}
      </div>
    </div>
  )
}
