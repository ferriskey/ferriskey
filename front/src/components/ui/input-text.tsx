import { cn } from '@/lib/utils'
import { Eye, EyeClosed } from 'lucide-react'
import { useRef, useState } from 'react'

export interface InputTextProps {
  name: string
  label: string
  value?: string | number
  type?: 'text' | 'number' | 'password' | 'email'
  className?: string
  onChange?: (value: string | number | undefined) => void
  error?: string
  disabled?: boolean
  autoComplete?: string
  // variable to control the toggle visibility of the password even if it's in disable
  togglePasswordVisibility?: boolean
}

export function InputText({
  name,
  label,
  value = '',
  onChange,
  type = 'text',
  error,
  className = '',
  disabled,
  autoComplete,
  togglePasswordVisibility = false,
}: InputTextProps) {
  const [focused, setFocused] = useState<boolean>(false)
  const inputRef = useRef<HTMLDivElement>(null)
  const currentValue = value
  const [currentType, setCurrentType] = useState<string>(type)

  const hasFocus = focused
  const hasLabelUp =
    hasFocus ||
    (currentValue?.toString() && currentValue?.toString().length > 0)
      ? 'input--label-up'
      : ''

  const hasError = error && error.length > 0 ? 'input--error' : ''

  const inputActions = hasFocus
    ? 'input--focused'
    : disabled && !togglePasswordVisibility
      ? 'input--disabled'
      : ''

  return (
    <div
      className={className}
      onClick={() => inputRef.current?.querySelector('input')?.focus()}
    >
      <div className='relative'>
        <div
          className={cn('input', inputActions, hasError, hasLabelUp)}
          ref={inputRef}
        >
          <div>
            <label
              htmlFor={label}
              className={cn(hasFocus ? 'text-xs' : 'translate-y-2 text-sm')}
            >
              {label}
            </label>

            <input
              name={name}
              id={label}
              className={'input__value'}
              type={currentType}
              disabled={disabled}
              autoComplete={autoComplete}
              value={currentValue}
              onChange={(e) => {
                if (!onChange) return

                if (type === 'number') {
                  const val = e.currentTarget.valueAsNumber
                  onChange(Number.isNaN(val) ? undefined : val)
                  return
                }

                onChange(e.currentTarget.value)
              }}
              onFocus={() => setFocused(true)}
              onBlur={() => setFocused(false)}
            />

            {String(currentValue).length > 0 && type === 'password' && (
              <div
                className='absolute right-4 top-1/2 -translate-y-1/2 text-muted-foreground transition-colors hover:text-muted-foreground'
                onClick={() => {
                  setCurrentType(
                    currentType === 'password' ? 'text' : 'password'
                  )

                }}
              >
                {currentType === 'password' && <Eye className='text-sm' />}
                {currentType !== 'password' && (
                  <EyeClosed className='text-sm' />
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {error && (
        <p className='mt-0.5 px-3 text-xs font-medium text-destructive'>{error}</p>
      )}
    </div>
  )
}
