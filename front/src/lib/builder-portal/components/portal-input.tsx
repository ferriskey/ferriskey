import { useId, useRef, useState, type CSSProperties } from 'react'
import { Eye, EyeClosed } from 'lucide-react'

interface PortalInputProps {
  name: string | undefined
  label: string
  type?: string
  placeholder?: string
  disabled?: boolean
  required?: boolean
  autoComplete?: string
  /**
   * When `true`, force the floating label into the lifted position even if
   * the field is empty. Useful for canvas previews where the input is
   * disabled and we still want to convey the floating-label affordance.
   */
  alwaysFloatLabel?: boolean
}

/**
 * Portal-facing text input that mirrors the admin design-system's
 * `InputText` look (floating label, ring on focus, password reveal toggle)
 * while pulling its colors / radius / border-weight from the portal theme
 * tokens (`--fk-color-*`, `--fk-radius-input`, `--fk-border-input`).
 *
 * Kept uncontrolled — the surrounding `<form>` reads the value via
 * `FormData` at submit time, same as the plain `<input>` it replaces. The
 * floating-label trick needs to know when the user typed something though,
 * so we keep a tiny `filled` state purely for the label position; the value
 * itself is not mirrored back to React state.
 */
export function PortalInput({
  name,
  label,
  type = 'text',
  placeholder,
  disabled,
  required,
  autoComplete,
  alwaysFloatLabel,
}: PortalInputProps) {
  const inputId = useId()
  const [focused, setFocused] = useState(false)
  const [filled, setFilled] = useState(false)
  // Password-reveal toggle. Mirrors the design-system behaviour: a small
  // eye icon appears once the user types into a `password` field. We track
  // the visibility flip with local state.
  const [revealed, setRevealed] = useState(false)
  const inputRef = useRef<HTMLInputElement | null>(null)

  const labelUp = focused || filled || alwaysFloatLabel
  const effectiveType = type === 'password' && revealed ? 'text' : type

  const wrapperStyle: CSSProperties = {
    position: 'relative',
    minHeight: 52,
    cursor: disabled ? 'not-allowed' : 'text',
    borderRadius: 'var(--fk-radius-input, 6px)',
    border: `var(--fk-border-input, 1px) solid ${
      focused
        ? 'var(--fk-color-primary-button, #635dff)'
        : 'rgba(0,0,0,0.18)'
    }`,
    backgroundColor: 'var(--fk-color-widget-bg, #ffffff)',
    paddingLeft: 12,
    paddingRight: type === 'password' ? 40 : 12,
    paddingTop: 6,
    paddingBottom: 6,
    transition: 'border-color 120ms cubic-bezier(0.25,0.1,0.25,1), box-shadow 120ms ease',
    boxShadow: focused
      ? '0 0 0 3px color-mix(in srgb, var(--fk-color-primary-button, #635dff) 22%, transparent)'
      : 'none',
    opacity: disabled ? 0.6 : 1,
  }

  const labelStyle: CSSProperties = {
    position: 'absolute',
    left: 12,
    top: labelUp ? 6 : 16,
    fontSize: labelUp ? 11 : 'var(--fk-font-input-label-size, 14px)',
    fontWeight: 'var(--fk-font-input-label-weight, 500)' as CSSProperties['fontWeight'],
    color: 'var(--fk-color-body-text, #374151)',
    opacity: labelUp ? 0.7 : 0.55,
    transition: 'top 160ms cubic-bezier(0.25,0.1,0.25,1), font-size 160ms cubic-bezier(0.25,0.1,0.25,1), opacity 160ms ease',
    pointerEvents: 'none',
    userSelect: 'none',
  }

  const inputStyle: CSSProperties = {
    width: '100%',
    border: 'none',
    outline: 'none',
    background: 'transparent',
    fontSize: 'var(--fk-font-base-size, 14px)',
    color: 'var(--fk-color-body-text, #111827)',
    paddingTop: 14,
    paddingBottom: 0,
    appearance: 'none',
  }

  return (
    <div
      style={wrapperStyle}
      onClick={() => inputRef.current?.focus()}
    >
      <label htmlFor={inputId} style={labelStyle}>
        {label}
        {required && (
          <span aria-hidden style={{ marginLeft: 2, color: 'var(--fk-color-error, #d03c38)' }}>
            *
          </span>
        )}
      </label>
      <input
        ref={inputRef}
        id={inputId}
        name={name}
        type={effectiveType}
        // Hide placeholder until the label has lifted, otherwise it would
        // sit under the label at rest and read as a double label.
        placeholder={labelUp ? placeholder : ''}
        disabled={disabled}
        required={required}
        autoComplete={autoComplete}
        style={inputStyle}
        onFocus={() => setFocused(true)}
        onBlur={(e) => {
          setFocused(false)
          setFilled(e.currentTarget.value.length > 0)
        }}
        onChange={(e) => {
          setFilled(e.currentTarget.value.length > 0)
        }}
      />
      {type === 'password' && (
        <button
          type='button'
          // Reveal toggle — only meaningful at runtime. In the canvas
          // preview the input is disabled, so we keep the button visible
          // but inert (matches the design-system disabled treatment).
          onClick={(e) => {
            e.stopPropagation()
            setRevealed((v) => !v)
          }}
          aria-label={revealed ? 'Hide password' : 'Show password'}
          disabled={disabled}
          style={{
            position: 'absolute',
            right: 8,
            top: '50%',
            transform: 'translateY(-50%)',
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: 28,
            height: 28,
            background: 'transparent',
            border: 'none',
            cursor: disabled ? 'not-allowed' : 'pointer',
            color: 'var(--fk-color-body-text, #6b7280)',
            opacity: 0.6,
          }}
        >
          {revealed ? <EyeClosed size={16} /> : <Eye size={16} />}
        </button>
      )}
    </div>
  )
}
