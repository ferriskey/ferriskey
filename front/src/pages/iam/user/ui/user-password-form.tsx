import { Lock, TimerReset } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { ChoiceCards, FieldRow, type Choice } from '@/components/kit'

export type PasswordValidity = 'temporary' | 'permanent'

const PASSWORD_MASK = '••••••••••••'

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
  const { t } = useTranslation('user')
  const touched = password.length > 0 || confirmPassword.length > 0

  const validityChoices: Choice<PasswordValidity>[] = [
    {
      value: 'temporary',
      label: t('detail.credentials.password.validity.temporary.label'),
      description: t('detail.credentials.password.validity.temporary.description'),
      icon: TimerReset,
    },
    {
      value: 'permanent',
      label: t('detail.credentials.password.validity.permanent.label'),
      description: t('detail.credentials.password.validity.permanent.description'),
      icon: Lock,
    },
  ]

  return (
    <>
      <FieldRow
        label={t('detail.credentials.password.validity.label')}
        description={t('detail.credentials.password.validity.description')}
      >
        <ChoiceCards
          label={t('detail.credentials.password.validity.group_label')}
          value={validity}
          onChange={onValidityChange}
          options={validityChoices}
        />
      </FieldRow>

      <FieldRow
        label={t('detail.credentials.password.new.label')}
        description={t('detail.credentials.password.new.description')}
        htmlFor='user-new-password'
      >
        <div className='max-w-sm space-y-2'>
          <Input
            id='user-new-password'
            type='password'
            autoComplete='new-password'
            value={password}
            onChange={(e) => onPasswordChange(e.target.value)}
            placeholder={PASSWORD_MASK}
            aria-invalid={Boolean(errors.password)}
          />
          {errors.password && <p className='text-xs text-fk-danger'>{errors.password}</p>}
        </div>
      </FieldRow>

      <FieldRow
        label={t('detail.credentials.password.confirm.label')}
        description={t('detail.credentials.password.confirm.description')}
        htmlFor='user-confirm-password'
      >
        <div className='max-w-sm space-y-3'>
          <Input
            id='user-confirm-password'
            type='password'
            autoComplete='new-password'
            value={confirmPassword}
            onChange={(e) => onConfirmPasswordChange(e.target.value)}
            placeholder={PASSWORD_MASK}
            aria-invalid={Boolean(errors.confirmPassword)}
          />
          {errors.confirmPassword && (
            <p className='text-xs text-fk-danger'>{errors.confirmPassword}</p>
          )}
          <div className='flex gap-2'>
            <Button size='sm' disabled={!canSubmit} onClick={onSubmit}>
              {hasPassword
                ? t('detail.credentials.password.submit_replace')
                : t('detail.credentials.password.submit_set')}
            </Button>
            {touched && (
              <Button variant='ghost' size='sm' onClick={onReset}>
                {t('detail.credentials.password.cancel')}
              </Button>
            )}
          </div>
        </div>
      </FieldRow>
    </>
  )
}
