import { useMemo, type CSSProperties } from 'react'
import { AlertCircle } from 'lucide-react'
import { usePortalThemeContext } from '../context/portal-theme-context'
import { themeToCssVars } from '../lib/theme'

/**
 * Mock auth screen rendered alongside the theme builder. Every visible
 * element is wired to a CSS variable so any token change shows up live —
 * the previous version only exercised primary buttons / inputs / links and
 * silently hid social, secondary, error, subtitle, body, and spacing
 * tokens (admins couldn't tell whether their changes had any effect).
 */

const widgetStyle: CSSProperties = {
  background: 'var(--fk-color-widget-bg)',
  color: 'var(--fk-color-body-text)',
  borderRadius: 'var(--fk-radius-widget)',
  borderWidth: 'var(--fk-border-widget)',
  borderStyle: 'solid',
  borderColor: 'rgba(0,0,0,0.12)',
  boxShadow: 'var(--fk-shadow-widget)',
  fontSize: 'var(--fk-font-body-size)',
  fontWeight: 'var(--fk-font-body-weight)' as CSSProperties['fontWeight'],
  // `widgetPadding` controls the inner gutter; switching from `p-8` to a
  // token-driven padding lets admins actually see the result of the slider.
  padding: 'var(--fk-spacing-widget-padding)',
}

const titleStyle: CSSProperties = {
  fontSize: 'var(--fk-font-title-size)',
  fontWeight: 'var(--fk-font-title-weight)' as CSSProperties['fontWeight'],
  lineHeight: 1.2,
  margin: 0,
}

const subtitleStyle: CSSProperties = {
  fontSize: 'var(--fk-font-subtitle-size)',
  fontWeight: 'var(--fk-font-subtitle-weight)' as CSSProperties['fontWeight'],
  color: 'var(--fk-color-body-text)',
  opacity: 0.7,
  margin: 0,
}

const labelStyle: CSSProperties = {
  fontSize: 'var(--fk-font-input-label-size)',
  fontWeight: 'var(--fk-font-input-label-weight)' as CSSProperties['fontWeight'],
}

const inputStyle: CSSProperties = {
  borderRadius: 'var(--fk-radius-input)',
  borderWidth: 'var(--fk-border-input)',
  borderStyle: 'solid',
  borderColor: 'rgba(0,0,0,0.15)',
  background: 'transparent',
  color: 'var(--fk-color-body-text)',
}

const primaryButtonStyle: CSSProperties = {
  background: 'var(--fk-color-primary-button)',
  color: 'var(--fk-color-primary-button-label)',
  borderRadius: 'var(--fk-radius-button)',
  borderWidth: 'var(--fk-border-button)',
  borderStyle: 'solid',
  borderColor: 'transparent',
  fontSize: 'var(--fk-font-button-size)',
  fontWeight: 'var(--fk-font-button-weight)' as CSSProperties['fontWeight'],
}

const secondaryButtonStyle: CSSProperties = {
  background: 'var(--fk-color-secondary-button)',
  color: 'var(--fk-color-secondary-button-label)',
  borderRadius: 'var(--fk-radius-button)',
  borderWidth: 'var(--fk-border-button)',
  borderStyle: 'solid',
  borderColor: 'rgba(0,0,0,0.12)',
  fontSize: 'var(--fk-font-button-size)',
  fontWeight: 'var(--fk-font-button-weight)' as CSSProperties['fontWeight'],
}

const socialButtonStyle: CSSProperties = {
  background: 'var(--fk-color-social-button-bg)',
  color: 'var(--fk-color-social-button-label)',
  borderRadius: 'var(--fk-radius-button)',
  borderWidth: 'var(--fk-border-social-button)',
  borderStyle: 'solid',
  borderColor: 'var(--fk-color-social-button-border)',
  fontSize: 'var(--fk-font-button-size)',
  fontWeight: 'var(--fk-font-button-weight)' as CSSProperties['fontWeight'],
}

const linkStyle: CSSProperties = {
  color: 'var(--fk-color-links)',
  fontSize: 'var(--fk-font-link-size)',
  fontWeight: 'var(--fk-font-link-weight)' as CSSProperties['fontWeight'],
  textDecoration: 'var(--fk-font-link-decoration)' as CSSProperties['textDecoration'],
}

const errorBannerStyle: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: 8,
  padding: '10px 12px',
  borderRadius: 'var(--fk-radius-input)',
  border: '1px solid var(--fk-color-error)',
  background: 'color-mix(in srgb, var(--fk-color-error) 10%, transparent)',
  color: 'var(--fk-color-error)',
  fontSize: 'var(--fk-font-body-size)',
}

const separatorStyle: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: 8,
  fontSize: 'var(--fk-font-body-size)',
  color: 'var(--fk-color-body-text)',
  opacity: 0.6,
}

const separatorLineStyle: CSSProperties = {
  flex: 1,
  height: 1,
  background: 'currentColor',
  opacity: 0.3,
}

const SOCIAL_PROVIDERS = [
  { id: 'google', label: 'Continue with Google', glyph: 'G' },
  { id: 'github', label: 'Continue with GitHub', glyph: 'GH' },
]

export function PreviewCard() {
  const { theme } = usePortalThemeContext()
  const vars = useMemo(() => themeToCssVars(theme), [theme])

  const wrapperStyle: CSSProperties = {
    ...vars,
    background: 'var(--fk-color-page-bg)',
    fontFamily: 'inherit',
  }

  const formStyle: CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    // `fieldGap` between consecutive inputs (the rows of the form), then
    // `sectionGap` between distinct groups (form → social → footer) below.
    gap: 'var(--fk-spacing-field-gap)',
  }

  const groupStyle: CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: 'var(--fk-spacing-section-gap)',
  }

  return (
    <div
      style={wrapperStyle}
      className='flex h-full w-full items-center justify-center overflow-auto p-10'
    >
      <div style={widgetStyle} className='w-full max-w-sm'>
        <div style={groupStyle}>
          <div className='flex flex-col items-center gap-2 text-center'>
            <div className='h-9 w-9 rounded bg-muted' aria-hidden />
            <h3 style={titleStyle}>Welcome back</h3>
            <p style={subtitleStyle}>Sign in to your workspace.</p>
          </div>

          {/* Error banner exercises `--fk-color-error` so the admin can see
              the impact of recolouring the destructive token. */}
          <div style={errorBannerStyle}>
            <AlertCircle size={16} aria-hidden />
            <span>Invalid email or password.</span>
          </div>

          <div style={formStyle}>
            <div className='flex flex-col gap-1.5'>
              <label style={labelStyle}>Email</label>
              <input
                style={inputStyle}
                className='h-10 px-3 text-sm outline-none'
                placeholder='you@example.com'
                readOnly
              />
            </div>
            <div className='flex flex-col gap-1.5'>
              <label style={labelStyle}>Password</label>
              <input
                style={inputStyle}
                className='h-10 px-3 text-sm outline-none'
                type='password'
                value='••••••••'
                readOnly
              />
            </div>

            <button
              type='button'
              style={primaryButtonStyle}
              className='h-10 w-full px-4 transition-opacity hover:opacity-95'
            >
              Continue
            </button>

            <button
              type='button'
              style={secondaryButtonStyle}
              className='h-10 w-full px-4 transition-opacity hover:opacity-95'
            >
              Use another account
            </button>
          </div>

          <div style={separatorStyle}>
            <span style={separatorLineStyle} />
            <span>or continue with</span>
            <span style={separatorLineStyle} />
          </div>

          <div className='flex flex-col gap-2'>
            {SOCIAL_PROVIDERS.map((p) => (
              <button
                key={p.id}
                type='button'
                style={socialButtonStyle}
                className='flex h-10 w-full items-center justify-center gap-3 px-3 transition-opacity hover:opacity-95'
              >
                <span
                  aria-hidden
                  className='flex h-5 w-5 items-center justify-center rounded-full bg-current/10 text-[11px] font-semibold opacity-80'
                  style={{ background: 'rgba(0,0,0,0.06)' }}
                >
                  {p.glyph}
                </span>
                <span className='flex-1 text-left'>{p.label}</span>
              </button>
            ))}
          </div>

          <div className='flex justify-center'>
            <a href='#' style={linkStyle} onClick={(e) => e.preventDefault()}>
              Forgot password?
            </a>
          </div>
        </div>
      </div>
    </div>
  )
}
