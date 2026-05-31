import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { defaultTheme, usePortalThemeContext } from '../../context/portal-theme-context'
import type { ThemeShadow } from '../../lib/theme'
import { ControlRow } from '../controls/control-row'
import { ColorPicker } from '../controls/color-picker'
import { ValueSlider } from '../controls/value-slider'
import { PanelHeader, PanelSection } from './section'

const D = defaultTheme

export function WidgetPanel() {
  const { theme, setColor, setBorder, setSpacing } = usePortalThemeContext()
  const { colors, borders, spacing } = theme

  return (
    <div className='flex flex-col'>
      <PanelHeader
        title='Widget'
        description='The card that wraps every auth flow — Login, Register, Magic link, etc.'
      />

      <PanelSection title='Surface'>
        <ColorPicker
          label='Background'
          value={colors.widgetBackground}
          defaultValue={D.colors.widgetBackground}
          onChange={(v) => setColor('widgetBackground', v)}
        />
      </PanelSection>

      <PanelSection title='Shape'>
        <ValueSlider
          label='Radius'
          value={borders.widgetRadius}
          defaultValue={D.borders.widgetRadius}
          onChange={(v) => setBorder('widgetRadius', v)}
          min={0}
          max={32}
          unit='px'
        />
        <ValueSlider
          label='Border'
          value={borders.widgetBorderWeight}
          defaultValue={D.borders.widgetBorderWeight}
          onChange={(v) => setBorder('widgetBorderWeight', v)}
          min={0}
          max={6}
          unit='px'
        />
        <ControlRow
          label='Shadow'
          modified={borders.widgetShadow !== D.borders.widgetShadow}
          onReset={() => setBorder('widgetShadow', D.borders.widgetShadow)}
        >
          <Select
            value={borders.widgetShadow}
            onValueChange={(value: ThemeShadow) => setBorder('widgetShadow', value)}
          >
            <SelectTrigger className='h-7 w-full text-xs'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value='none'>None</SelectItem>
              <SelectItem value='small'>Small</SelectItem>
              <SelectItem value='large'>Large</SelectItem>
            </SelectContent>
          </Select>
        </ControlRow>
      </PanelSection>

      <PanelSection title='Spacing'>
        <ValueSlider
          label='Padding'
          value={spacing.widgetPadding}
          defaultValue={D.spacing.widgetPadding}
          onChange={(v) => setSpacing('widgetPadding', v)}
          min={0}
          max={64}
          unit='px'
        />
      </PanelSection>
    </div>
  )
}
