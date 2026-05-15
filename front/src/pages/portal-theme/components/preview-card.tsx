import { useMemo, type CSSProperties } from 'react'
import { usePortalThemeContext } from '../context/portal-theme-context'
import { themeToCssVars } from '../lib/theme'

const widgetStyle: CSSProperties = {
  background: 'var(--fk-color-widget-bg)',
  color: 'var(--fk-color-body-text)',
  borderRadius: 'var(--fk-radius-widget)',
  borderWidth: 'var(--fk-border-widget)',
  borderStyle: 'solid',
  borderColor: 'rgba(0,0,0,0.08)',
  boxShadow: 'var(--fk-shadow-widget)',
  fontSize: 'var(--fk-font-body-size)',
  fontWeight: 'var(--fk-font-body-weight)' as CSSProperties['fontWeight'],
}

const titleStyle: CSSProperties = {
  fontSize: 'var(--fk-font-title-size)',
  fontWeight: 'var(--fk-font-title-weight)' as CSSProperties['fontWeight'],
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

const linkStyle: CSSProperties = {
  color: 'var(--fk-color-links)',
  fontSize: 'var(--fk-font-link-size)',
  fontWeight: 'var(--fk-font-link-weight)' as CSSProperties['fontWeight'],
  textDecoration: 'var(--fk-font-link-decoration)' as CSSProperties['textDecoration'],
}

export function PreviewCard() {
  const { theme } = usePortalThemeContext()
  const vars = useMemo(() => themeToCssVars(theme), [theme])

  const wrapperStyle: CSSProperties = {
    ...vars,
    background: 'var(--fk-color-page-bg)',
    fontFamily: 'inherit',
  }

  return (
    <div
      style={wrapperStyle}
      className='flex h-full w-full items-center justify-center overflow-auto p-10'
    >
      <div style={widgetStyle} className='w-full max-w-sm p-8'>
        <div className='mb-6 flex flex-col items-center gap-2'>
          <div className='h-9 w-9 rounded bg-muted' aria-hidden />
          <h3 style={titleStyle}>Welcome</h3>
          <p style={{ color: 'var(--fk-color-body-text)', opacity: 0.7 }}>Log in to continue</p>
        </div>

        <div className='flex flex-col gap-4'>
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
