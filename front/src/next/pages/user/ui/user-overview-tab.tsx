import { Check } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { FieldRow, Section, SwitchField } from '@/components/kit'
import { DangerZone } from '@/components/kit/danger-zone'
import { cn } from '@/lib/utils'
import { Schemas } from '@/api/api.client'

import User = Schemas.User
import RequiredAction = Schemas.RequiredAction

const requiredActionCatalogue: { value: RequiredAction; label: string; hint: string }[] = [
  {
    value: 'verify_email',
    label: 'Verify email',
    hint: 'A verification link is sent at the next sign-in.',
  },
  {
    value: 'update_password',
    label: 'Update password',
    hint: 'The current password must be replaced before any access.',
  },
  {
    value: 'configure_otp',
    label: 'Configure an OTP',
    hint: 'TOTP enrolment is enforced before any access.',
  },
  {
    value: 'configure_passkey',
    label: 'Register a passkey',
    hint: 'A WebAuthn key is requested at the next sign-in.',
  },
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
  const toggleAction = (action: RequiredAction) =>
    onRequiredActionsChange(
      requiredActions.includes(action)
        ? requiredActions.filter((a) => a !== action)
        : [...requiredActions, action]
    )

  return (
    <>
      <Section title='Identity'>
        <FieldRow
          label='Username'
          description='Login identifier, fixed at creation: the update endpoint does not accept a new username.'
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
          label='First and last name'
          description='Optional — a service account has none. Shown in the console and in issued tokens.'
        >
          <div className='flex max-w-sm gap-2'>
            <Input
              value={firstname}
              onChange={(e) => onFirstnameChange(e.target.value)}
              placeholder='First name'
              aria-label='First name'
            />
            <Input
              value={lastname}
              onChange={(e) => onLastnameChange(e.target.value)}
              placeholder='Last name'
              aria-label='Last name'
            />
          </div>
        </FieldRow>

        <FieldRow
          label='Email'
          description='Used for verification and password recovery.'
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
          label='User enabled'
          description='A disabled account can no longer obtain a token.'
        >
          <SwitchField checked={enabled} onCheckedChange={onEnabledChange} />
        </FieldRow>

        <FieldRow
          label='Email verified'
          description='Marks the address as verified without going through the verification mail.'
        >
          <SwitchField
            checked={emailVerified}
            onCheckedChange={onEmailVerifiedChange}
            onLabel='Verified'
            offLabel='Not verified'
          />
        </FieldRow>
      </Section>

      <Section
        title='Required actions'
        description={
          requiredActions.length > 0
            ? `${requiredActions.length} action${requiredActions.length > 1 ? 's' : ''} to complete at the next sign-in.`
            : 'No action enforced: the user signs in directly.'
        }
        contained={false}
      >
        <div className='grid gap-2 sm:grid-cols-2'>
          {requiredActionCatalogue.map((action) => {
            const on = requiredActions.includes(action.value)
            return (
              <button
                key={action.value}
                type='button'
                role='switch'
                aria-checked={on}
                onClick={() => toggleAction(action.value)}
                className={cn(
                  'flex cursor-pointer flex-col gap-0.5 rounded-md border p-3 text-left transition-colors',
                  'focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-fk-primary/30',
                  on
                    ? 'border-fk-primary-border bg-fk-primary-soft'
                    : 'border-fk-line bg-white hover:bg-neutral-50 dark:bg-neutral-900 dark:hover:bg-neutral-900'
                )}
              >
                <span className='flex items-center gap-2'>
                  <span
                    className={cn(
                      'grid size-4 shrink-0 place-items-center rounded border transition-colors',
                      on ? 'border-fk-primary bg-fk-primary text-white' : 'border-fk-line bg-white dark:bg-neutral-900'
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
                    {action.label}
                  </span>
                  <span className='flex-1' />
                  <span className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
                    {action.value}
                  </span>
                </span>
                <span
                  className={cn(
                    'pl-6 text-xs leading-relaxed',
                    on ? 'text-fk-primary-text/80' : 'text-neutral-500 dark:text-neutral-400'
                  )}
                >
                  {action.hint}
                </span>
              </button>
            )
          })}
        </div>
      </Section>

      <DangerZone
        resourceName={user.username}
        label='Delete this user'
        description='Once deleted, all associated sessions, credentials, and role assignments will be permanently removed.'
        buttonLabel='Delete user'
        confirmTitle='Delete user'
        confirmDescription={`This will permanently delete the user "${user.username}" and all associated data.`}
        onConfirm={onDelete}
      />
    </>
  )
}
