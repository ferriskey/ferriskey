import { useTranslation } from 'react-i18next'
import { defaultTheme } from '../../context/portal-theme-context'
import { usePortalThemeContext } from '../../context/portal-theme-context'
import { ColorPicker } from '../controls/color-picker'
import { ValueSlider } from '../controls/value-slider'
import { PanelHeader, PanelSection } from './section'
import { UNIT_PERCENT, UNIT_PX } from './units'

const D = defaultTheme

const TOKENS = {
  primaryBackground: 'primaryButton',
  primaryLabel: 'primaryButtonLabel',
  secondaryBackground: 'secondaryButton',
  secondaryLabel: 'secondaryButtonLabel',
  socialBackground: 'socialButtonBackground',
  socialLabel: 'socialButtonLabel',
  socialBorder: 'socialButtonBorder',
  socialBorderWeight: 'socialButtonBorderWeight',
  magicLinkBackground: 'magicLinkButtonBackground',
  magicLinkLabel: 'magicLinkButtonLabel',
  magicLinkBorder: 'magicLinkButtonBorder',
  magicLinkBorderWeight: 'magicLinkButtonBorderWeight',
  passkeyBackground: 'passkeyButtonBackground',
  passkeyLabel: 'passkeyButtonLabel',
  passkeyBorder: 'passkeyButtonBorder',
  passkeyBorderWeight: 'passkeyButtonBorderWeight',
  radius: 'buttonRadius',
  borderWeight: 'buttonBorderWeight',
  font: 'buttons',
} as const

export function ButtonsPanel() {
  const { t } = useTranslation('portal')
  const { theme, setColor, setBorder, setFont } = usePortalThemeContext()
  const { colors, borders, fonts } = theme

  return (
    <div className='flex flex-col'>
      <PanelHeader
        title={t('builder.panel.buttons.title')}
        description={t('builder.panel.buttons.description')}
      />

      <PanelSection title={t('builder.panel.buttons.section.primary')}>
        <ColorPicker
          label={t('builder.control.background')}
          value={colors.primaryButton}
          defaultValue={D.colors.primaryButton}
          onChange={(v) => setColor(TOKENS.primaryBackground, v)}
        />
        <ColorPicker
          label={t('builder.control.label')}
          value={colors.primaryButtonLabel}
          defaultValue={D.colors.primaryButtonLabel}
          onChange={(v) => setColor(TOKENS.primaryLabel, v)}
        />
      </PanelSection>

      <PanelSection title={t('builder.panel.buttons.section.secondary')}>
        <ColorPicker
          label={t('builder.control.background')}
          value={colors.secondaryButton}
          defaultValue={D.colors.secondaryButton}
          onChange={(v) => setColor(TOKENS.secondaryBackground, v)}
        />
        <ColorPicker
          label={t('builder.control.label')}
          value={colors.secondaryButtonLabel}
          defaultValue={D.colors.secondaryButtonLabel}
          onChange={(v) => setColor(TOKENS.secondaryLabel, v)}
        />
      </PanelSection>

      <PanelSection title={t('builder.panel.buttons.section.social')}>
        <ColorPicker
          label={t('builder.control.background')}
          value={colors.socialButtonBackground}
          defaultValue={D.colors.socialButtonBackground}
          onChange={(v) => setColor(TOKENS.socialBackground, v)}
        />
        <ColorPicker
          label={t('builder.control.label')}
          value={colors.socialButtonLabel}
          defaultValue={D.colors.socialButtonLabel}
          onChange={(v) => setColor(TOKENS.socialLabel, v)}
        />
        <ColorPicker
          label={t('builder.control.border_color')}
          value={colors.socialButtonBorder}
          defaultValue={D.colors.socialButtonBorder}
          onChange={(v) => setColor(TOKENS.socialBorder, v)}
        />
        <ValueSlider
          label={t('builder.control.border_width')}
          value={borders.socialButtonBorderWeight}
          defaultValue={D.borders.socialButtonBorderWeight}
          onChange={(v) => setBorder(TOKENS.socialBorderWeight, v)}
          min={0}
          max={6}
          unit={UNIT_PX}
        />
      </PanelSection>

      <PanelSection title={t('builder.panel.buttons.section.magic_link')} defaultOpen={false}>
        <ColorPicker
          label={t('builder.control.background')}
          value={colors.magicLinkButtonBackground}
          defaultValue={D.colors.magicLinkButtonBackground}
          onChange={(v) => setColor(TOKENS.magicLinkBackground, v)}
        />
        <ColorPicker
          label={t('builder.control.label')}
          value={colors.magicLinkButtonLabel}
          defaultValue={D.colors.magicLinkButtonLabel}
          onChange={(v) => setColor(TOKENS.magicLinkLabel, v)}
        />
        <ColorPicker
          label={t('builder.control.border_color')}
          value={colors.magicLinkButtonBorder}
          defaultValue={D.colors.magicLinkButtonBorder}
          onChange={(v) => setColor(TOKENS.magicLinkBorder, v)}
        />
        <ValueSlider
          label={t('builder.control.border_width')}
          value={borders.magicLinkButtonBorderWeight}
          defaultValue={D.borders.magicLinkButtonBorderWeight}
          onChange={(v) => setBorder(TOKENS.magicLinkBorderWeight, v)}
          min={0}
          max={6}
          unit={UNIT_PX}
        />
      </PanelSection>

      <PanelSection title={t('builder.panel.buttons.section.passkey')} defaultOpen={false}>
        <ColorPicker
          label={t('builder.control.background')}
          value={colors.passkeyButtonBackground}
          defaultValue={D.colors.passkeyButtonBackground}
          onChange={(v) => setColor(TOKENS.passkeyBackground, v)}
        />
        <ColorPicker
          label={t('builder.control.label')}
          value={colors.passkeyButtonLabel}
          defaultValue={D.colors.passkeyButtonLabel}
          onChange={(v) => setColor(TOKENS.passkeyLabel, v)}
        />
        <ColorPicker
          label={t('builder.control.border_color')}
          value={colors.passkeyButtonBorder}
          defaultValue={D.colors.passkeyButtonBorder}
          onChange={(v) => setColor(TOKENS.passkeyBorder, v)}
        />
        <ValueSlider
          label={t('builder.control.border_width')}
          value={borders.passkeyButtonBorderWeight}
          defaultValue={D.borders.passkeyButtonBorderWeight}
          onChange={(v) => setBorder(TOKENS.passkeyBorderWeight, v)}
          min={0}
          max={6}
          unit={UNIT_PX}
        />
      </PanelSection>

      <PanelSection title={t('builder.panel.buttons.section.shape')}>
        <ValueSlider
          label={t('builder.control.radius')}
          value={borders.buttonRadius}
          defaultValue={D.borders.buttonRadius}
          onChange={(v) => setBorder(TOKENS.radius, v)}
          min={0}
          max={32}
          unit={UNIT_PX}
        />
        <ValueSlider
          label={t('builder.control.border')}
          value={borders.buttonBorderWeight}
          defaultValue={D.borders.buttonBorderWeight}
          onChange={(v) => setBorder(TOKENS.borderWeight, v)}
          min={0}
          max={6}
          unit={UNIT_PX}
        />
      </PanelSection>

      <PanelSection title={t('builder.panel.buttons.section.typography')} defaultOpen={false}>
        <ValueSlider
          label={t('builder.control.weight')}
          value={fonts.buttons.weight}
          defaultValue={D.fonts.buttons.weight}
          onChange={(weight) => setFont(TOKENS.font, { ...fonts.buttons, weight })}
          min={100}
          max={900}
          step={100}
        />
        <ValueSlider
          label={t('builder.control.size')}
          value={fonts.buttons.sizePct}
          defaultValue={D.fonts.buttons.sizePct}
          onChange={(sizePct) => setFont(TOKENS.font, { ...fonts.buttons, sizePct })}
          min={50}
          max={200}
          step={2.5}
          unit={UNIT_PERCENT}
        />
      </PanelSection>
    </div>
  )
}
