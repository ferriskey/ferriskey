import { defaultTheme } from '../../context/portal-theme-context'
import { usePortalThemeContext } from '../../context/portal-theme-context'
import { ColorPicker } from '../controls/color-picker'
import { ValueSlider } from '../controls/value-slider'
import { PanelHeader, PanelSection } from './section'

const D = defaultTheme

export function ButtonsPanel() {
  const { theme, setColor, setBorder, setFont } = usePortalThemeContext()
  const { colors, borders, fonts } = theme

  return (
    <div className='flex flex-col'>
      <PanelHeader
        title='Buttons'
        description='Primary and secondary CTAs, including their shape and typography.'
      />

      <PanelSection title='Primary'>
        <ColorPicker
          label='Background'
          value={colors.primaryButton}
          defaultValue={D.colors.primaryButton}
          onChange={(v) => setColor('primaryButton', v)}
        />
        <ColorPicker
          label='Label'
          value={colors.primaryButtonLabel}
          defaultValue={D.colors.primaryButtonLabel}
          onChange={(v) => setColor('primaryButtonLabel', v)}
        />
      </PanelSection>

      <PanelSection title='Secondary'>
        <ColorPicker
          label='Background'
          value={colors.secondaryButton}
          defaultValue={D.colors.secondaryButton}
          onChange={(v) => setColor('secondaryButton', v)}
        />
        <ColorPicker
          label='Label'
          value={colors.secondaryButtonLabel}
          defaultValue={D.colors.secondaryButtonLabel}
          onChange={(v) => setColor('secondaryButtonLabel', v)}
        />
      </PanelSection>

      <PanelSection title='Social (Google, GitHub, …)'>
        <ColorPicker
          label='Background'
          value={colors.socialButtonBackground}
          defaultValue={D.colors.socialButtonBackground}
          onChange={(v) => setColor('socialButtonBackground', v)}
        />
        <ColorPicker
          label='Label'
          value={colors.socialButtonLabel}
          defaultValue={D.colors.socialButtonLabel}
          onChange={(v) => setColor('socialButtonLabel', v)}
        />
        <ColorPicker
          label='Border color'
          value={colors.socialButtonBorder}
          defaultValue={D.colors.socialButtonBorder}
          onChange={(v) => setColor('socialButtonBorder', v)}
        />
        <ValueSlider
          label='Border width'
          value={borders.socialButtonBorderWeight}
          defaultValue={D.borders.socialButtonBorderWeight}
          onChange={(v) => setBorder('socialButtonBorderWeight', v)}
          min={0}
          max={6}
          unit='px'
        />
      </PanelSection>

      <PanelSection title='Magic link' defaultOpen={false}>
        <ColorPicker
          label='Background'
          value={colors.magicLinkButtonBackground}
          defaultValue={D.colors.magicLinkButtonBackground}
          onChange={(v) => setColor('magicLinkButtonBackground', v)}
        />
        <ColorPicker
          label='Label'
          value={colors.magicLinkButtonLabel}
          defaultValue={D.colors.magicLinkButtonLabel}
          onChange={(v) => setColor('magicLinkButtonLabel', v)}
        />
        <ColorPicker
          label='Border color'
          value={colors.magicLinkButtonBorder}
          defaultValue={D.colors.magicLinkButtonBorder}
          onChange={(v) => setColor('magicLinkButtonBorder', v)}
        />
        <ValueSlider
          label='Border width'
          value={borders.magicLinkButtonBorderWeight}
          defaultValue={D.borders.magicLinkButtonBorderWeight}
          onChange={(v) => setBorder('magicLinkButtonBorderWeight', v)}
          min={0}
          max={6}
          unit='px'
        />
      </PanelSection>

      <PanelSection title='Passkey' defaultOpen={false}>
        <ColorPicker
          label='Background'
          value={colors.passkeyButtonBackground}
          defaultValue={D.colors.passkeyButtonBackground}
          onChange={(v) => setColor('passkeyButtonBackground', v)}
        />
        <ColorPicker
          label='Label'
          value={colors.passkeyButtonLabel}
          defaultValue={D.colors.passkeyButtonLabel}
          onChange={(v) => setColor('passkeyButtonLabel', v)}
        />
        <ColorPicker
          label='Border color'
          value={colors.passkeyButtonBorder}
          defaultValue={D.colors.passkeyButtonBorder}
          onChange={(v) => setColor('passkeyButtonBorder', v)}
        />
        <ValueSlider
          label='Border width'
          value={borders.passkeyButtonBorderWeight}
          defaultValue={D.borders.passkeyButtonBorderWeight}
          onChange={(v) => setBorder('passkeyButtonBorderWeight', v)}
          min={0}
          max={6}
          unit='px'
        />
      </PanelSection>

      <PanelSection title='Shape'>
        <ValueSlider
          label='Radius'
          value={borders.buttonRadius}
          defaultValue={D.borders.buttonRadius}
          onChange={(v) => setBorder('buttonRadius', v)}
          min={0}
          max={32}
          unit='px'
        />
        <ValueSlider
          label='Border'
          value={borders.buttonBorderWeight}
          defaultValue={D.borders.buttonBorderWeight}
          onChange={(v) => setBorder('buttonBorderWeight', v)}
          min={0}
          max={6}
          unit='px'
        />
      </PanelSection>

      <PanelSection title='Typography' defaultOpen={false}>
        <ValueSlider
          label='Weight'
          value={fonts.buttons.weight}
          defaultValue={D.fonts.buttons.weight}
          onChange={(weight) => setFont('buttons', { ...fonts.buttons, weight })}
          min={100}
          max={900}
          step={100}
        />
        <ValueSlider
          label='Size'
          value={fonts.buttons.sizePct}
          defaultValue={D.fonts.buttons.sizePct}
          onChange={(sizePct) => setFont('buttons', { ...fonts.buttons, sizePct })}
          min={50}
          max={200}
          step={2.5}
          unit='%'
        />
      </PanelSection>
    </div>
  )
}
