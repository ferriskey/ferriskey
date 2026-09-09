import { Lock, TimerReset } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { ChoiceCards, FieldRow, type Choice } from '@/components/kit'

export type PasswordValidity = 'temporary' | 'permanent'

const validityChoices: Choice<PasswordValidity>[] = [
  {
    value: 'temporary',
    label: 'Temporary',
    description: 'The user must change it at the next sign-in.',
    icon: TimerReset,
  },
  {
    value: 'permanent',
    label: 'Permanent',
    description: 'It stays valid until the user changes it.',
    icon: Lock,
  },
]

export interface UserPasswordFormProps {
  hasPassword: boolean
  password: string
  confirmPassword: string
  validity: PasswordValidity
  errors: { password?: string; confirmPassword?: string }
  canSubmit: boolean
  onPasswordChange: (v: string) => void
  onConfirmPasswordChange: (v: string) => void
  onValidityChange: (v: PasswordValidity) => void
  onSubmit: () => void
  onReset: () => void
}

export default function UserPasswordForm({
  hasPassword,
  password,
  confirmPassword,
  validity,
  errors,
  canSubmit,
  onPasswordChange,
  onConfirmPasswordChange,
  onValidityChange,
  onSubmit,
  onReset,
}: UserPasswordFormProps) {
  const touched = password.length > 0 || confirmPassword.length > 0

  return (
    <>
      <FieldRow
        label='Validity'
        description='Decides whether the password entered below must be changed at its first use.'
      >
        <ChoiceCards
          label='Password validity'
          value={validity}
          onChange={onValidityChange}
          options={validityChoices}
        />
      </FieldRow>

      <FieldRow
        label='New password'
        description='The stored password is never displayed: it can only be replaced.'
        htmlFor='user-new-password'
      >
        <div className='max-w-sm space-y-2'>
          <Input
            id='user-new-password'
            type='password'
            autoComplete='new-password'
            value={password}
            onChange={(e) => onPasswordChange(e.target.value)}
            placeholder='••••••••••••'
            aria-invalid={Boolean(errors.password)}
          />
          {errors.password && <p className='text-xs text-fk-danger'>{errors.password}</p>}
        </div>
      </FieldRow>

      <FieldRow
        label='Confirm password'
        description='Both entries must match before the password can be applied.'
        htmlFor='user-confirm-password'
      >
        <div className='max-w-sm space-y-3'>
          <Input
            id='user-confirm-password'
            type='password'
            autoComplete='new-password'
            value={confirmPassword}
            onChange={(e) => onConfirmPasswordChange(e.target.value)}
            placeholder='••••••••••••'
            aria-invalid={Boolean(errors.confirmPassword)}
          />
          {errors.confirmPassword && (
            <p className='text-xs text-fk-danger'>{errors.confirmPassword}</p>
          )}
          <div className='flex gap-2'>
            <Button size='sm' disabled={!canSubmit} onClick={onSubmit}>
              {hasPassword ? 'Replace the password' : 'Set the password'}
            </Button>
            {touched && (
              <Button variant='ghost' size='sm' onClick={onReset}>
                Cancel
              </Button>
            )}
          </div>
        </div>
      </FieldRow>
    </>
  )
}
