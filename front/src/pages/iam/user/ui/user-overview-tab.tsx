import { Check } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { FieldRow, Section, SwitchField } from '@/components/kit'
import { DangerZone } from '@/components/kit/danger-zone'
import { cn } from '@/lib/utils'
import { Schemas } from '@/api/api.client'

import User = Schemas.User
import RequiredAction = Schemas.RequiredAction

const REQUIRED_ACTIONS: RequiredAction[] = [
  'verify_email',
  'update_password',
  'configure_otp',
  'configure_passkey',
]

export interface UserOverviewTabProps {
  user: User
  username: string
  firstname: string
  lastname: string
  email: string
  enabled: boolean
  emailVerified: boolean
  requiredActions: RequiredAction[]
  emailError?: string
  onFirstnameChange: (v: string) => void
  onLastnameChange: (v: string) => void
  onEmailChange: (v: string) => void
  onEnabledChange: (v: boolean) => void
  onEmailVerifiedChange: (v: boolean) => void
  onRequiredActionsChange: (next: RequiredAction[]) => void
  onDelete: () => void
}

export default function UserOverviewTab({
  user,
  username,
  firstname,
  lastname,
  email,
  enabled,
  emailVerified,
  requiredActions,
  emailError,
  onFirstnameChange,
  onLastnameChange,
  onEmailChange,
  onEnabledChange,
  onEmailVerifiedChange,
  onRequiredActionsChange,
  onDelete,
}: UserOverviewTabProps) {
  const { t } = useTranslation('user')

  const toggleAction = (action: RequiredAction) =>
    onRequiredActionsChange(
      requiredActions.includes(action)
        ? requiredActions.filter((a) => a !== action)
        : [...requiredActions, action]
    )

  return (
    <>
      <Section title={t('detail.overview.identity.title')}>
        <FieldRow
          label={t('detail.overview.identity.username.label')}
          description={t('detail.overview.identity.username.description')}
          htmlFor='user-username'
        >
          <Input
            id='user-username'
            value={username}
            disabled
            className='max-w-sm'
          />
        </FieldRow>

        <FieldRow
          label={t('detail.overview.identity.name.label')}
          description={t('detail.overview.identity.name.description')}
        >
          <div className='flex max-w-sm gap-2'>
            <Input
              value={firstname}
              onChange={(e) => onFirstnameChange(e.target.value)}
              placeholder={t('detail.overview.identity.name.firstname_placeholder')}
              aria-label={t('detail.overview.identity.name.firstname_placeholder')}
            />
            <Input
              value={lastname}
              onChange={(e) => onLastnameChange(e.target.value)}
              placeholder={t('detail.overview.identity.name.lastname_placeholder')}
              aria-label={t('detail.overview.identity.name.lastname_placeholder')}
            />
          </div>
        </FieldRow>

        <FieldRow
          label={t('detail.overview.identity.email.label')}
          description={t('detail.overview.identity.email.description')}
          htmlFor='user-email'
        >
          <Input
            id='user-email'
            type='email'
            value={email}
            onChange={(e) => onEmailChange(e.target.value)}
            className='max-w-sm'
            aria-invalid={Boolean(emailError)}
          />
          {emailError && <p className='mt-1.5 text-xs text-fk-danger'>{emailError}</p>}
        </FieldRow>

        <FieldRow
          label={t('detail.overview.identity.enabled.label')}
          description={t('detail.overview.identity.enabled.description')}
        >
          <SwitchField checked={enabled} onCheckedChange={onEnabledChange} />
        </FieldRow>

        <FieldRow
          label={t('detail.overview.identity.email_verified.label')}
          description={t('detail.overview.identity.email_verified.description')}
        >
          <SwitchField
            checked={emailVerified}
            onCheckedChange={onEmailVerifiedChange}
            onLabel={t('detail.overview.identity.email_verified.switch_on')}
            offLabel={t('detail.overview.identity.email_verified.switch_off')}
          />
        </FieldRow>
      </Section>

      <Section
        title={t('detail.overview.required_actions.title')}
        description={
          requiredActions.length > 0
            ? t('detail.overview.required_actions.description', {
                count: requiredActions.length,
              })
            : t('detail.overview.required_actions.empty_description')
        }
        contained={false}
      >
        <div className='grid gap-2 sm:grid-cols-2'>
          {REQUIRED_ACTIONS.map((action) => {
            const on = requiredActions.includes(action)
            return (
              <button
                key={action}
                type='button'
                role='switch'
                aria-checked={on}
                onClick={() => toggleAction(action)}
                className={cn(
                  'flex cursor-pointer flex-col gap-0.5 rounded-md border p-3 text-left transition-colors',
                  'focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-fk-primary/30',
                  on
                    ? 'border-fk-primary-border bg-fk-primary-soft'
                    : 'border-fk-line bg-white hover:bg-neutral-50 dark:bg-fk-surface dark:hover:bg-fk-surface'
                )}
              >
                <span className='flex items-center gap-2'>
                  <span
                    className={cn(
                      'grid size-4 shrink-0 place-items-center rounded border transition-colors',
                      on ? 'border-fk-primary bg-fk-primary text-white' : 'border-fk-line bg-white dark:bg-fk-surface'
                    )}
                  >
                    {on && <Check className='size-2.5' strokeWidth={3} />}
                  </span>
                  <span
                    className={cn(
                      'text-xs font-medium',
                      on ? 'text-fk-primary-text' : 'text-neutral-900 dark:text-neutral-100'
                    )}
                  >
                    {t(`detail.overview.required_actions.options.${action}.label`)}
                  </span>
                  <span className='flex-1' />
                  <span className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
                    {action}
                  </span>
                </span>
                <span
                  className={cn(
                    'pl-6 text-xs leading-relaxed',
                    on ? 'text-fk-primary-text/80' : 'text-neutral-500 dark:text-neutral-400'
                  )}
                >
                  {t(`detail.overview.required_actions.options.${action}.hint`)}
                </span>
              </button>
            )
          })}
        </div>
      </Section>

      <DangerZone
        resourceName={user.username}
        label={t('detail.overview.danger.label')}
        description={t('detail.overview.danger.description')}
        buttonLabel={t('detail.overview.danger.button')}
        confirmTitle={t('detail.overview.danger.confirm_title')}
        confirmDescription={t('detail.overview.danger.confirm_description', {
          username: user.username,
        })}
        onConfirm={onDelete}
      />
    </>
  )
}
