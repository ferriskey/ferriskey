import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import SaveBar from '@/components/kit/save-bar'
import { FieldRow, PageShell, Section, SwitchField } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'

export interface PageCreateUserProps {
  username: string
  firstname: string
  lastname: string
  email: string
  emailVerified: boolean
  errors: { username?: string; email?: string }
  canSubmit: boolean
  onUsernameChange: (v: string) => void
  onFirstnameChange: (v: string) => void
  onLastnameChange: (v: string) => void
  onEmailChange: (v: string) => void
  onEmailVerifiedChange: (v: boolean) => void
  onBack: () => void
  onSubmit: () => void
}

export default function PageCreateUser({
  username,
  firstname,
  lastname,
  email,
  emailVerified,
  errors,
  canSubmit,
  onUsernameChange,
  onFirstnameChange,
  onLastnameChange,
  onEmailChange,
  onEmailVerifiedChange,
  onBack,
  onSubmit,
}: PageCreateUserProps) {
  return (
    <PageShell>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        Users
      </Button>

      <div className='pb-3'>
        <h1 className={tokens.header.title}>New user</h1>
        <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
          The account is created without any credential — set a password from its Credentials
          tab afterwards.
        </p>
      </div>

      <div className={tokens.page.blockGap}>
        <Section title='Identity'>
          <FieldRow
            label='Username'
            description='Login identifier, unique within the realm. It cannot be changed afterwards.'
            htmlFor='new-user-username'
          >
            <Input
              id='new-user-username'
              value={username}
              onChange={(e) => onUsernameChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(errors.username)}
            />
            {errors.username && (
              <p className='mt-1.5 text-xs text-fk-danger'>{errors.username}</p>
            )}
          </FieldRow>

          <FieldRow
            label='First and last name'
            description='Optional — shown in the console and in the tokens issued for this account.'
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
            description='Used for verification and password recovery. Leave empty if the account never receives mail.'
            htmlFor='new-user-email'
          >
            <Input
              id='new-user-email'
              type='email'
              value={email}
              onChange={(e) => onEmailChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(errors.email)}
            />
            {errors.email && <p className='mt-1.5 text-xs text-fk-danger'>{errors.email}</p>}
          </FieldRow>

          <FieldRow
            label='Email verified'
            description='Marks the address as already verified, so no verification is asked at first sign-in.'
          >
            <SwitchField
              checked={emailVerified}
              onCheckedChange={onEmailVerifiedChange}
              onLabel='Verified'
              offLabel='Unverified'
            />
          </FieldRow>
        </Section>
      </div>

      <SaveBar
        show={canSubmit}
        title='Create user'
        description='The account is created enabled, with no credential attached.'
        onCancel={onBack}
        actions={[{ label: 'Create user', onClick: onSubmit }]}
      />
    </PageShell>
  )
}
