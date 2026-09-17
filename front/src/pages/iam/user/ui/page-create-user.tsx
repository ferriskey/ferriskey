import { ArrowLeft } from 'lucide-react'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation('user')

  return (
    <PageShell>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        {t('create.back')}
      </Button>

      <div className='pb-3'>
        <h1 className={tokens.header.title}>{t('create.title')}</h1>
        <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
          {t('create.description')}
        </p>
      </div>

      <div className={tokens.page.blockGap}>
        <Section title={t('create.identity.title')}>
          <FieldRow
            label={t('create.identity.username.label')}
            description={t('create.identity.username.description')}
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
            label={t('create.identity.name.label')}
            description={t('create.identity.name.description')}
          >
            <div className='flex max-w-sm gap-2'>
              <Input
                value={firstname}
                onChange={(e) => onFirstnameChange(e.target.value)}
                placeholder={t('create.identity.name.firstname_placeholder')}
                aria-label={t('create.identity.name.firstname_placeholder')}
              />
              <Input
                value={lastname}
                onChange={(e) => onLastnameChange(e.target.value)}
                placeholder={t('create.identity.name.lastname_placeholder')}
                aria-label={t('create.identity.name.lastname_placeholder')}
              />
            </div>
          </FieldRow>

          <FieldRow
            label={t('create.identity.email.label')}
            description={t('create.identity.email.description')}
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
            label={t('create.identity.email_verified.label')}
            description={t('create.identity.email_verified.description')}
          >
            <SwitchField
              checked={emailVerified}
              onCheckedChange={onEmailVerifiedChange}
              onLabel={t('create.identity.email_verified.switch_on')}
              offLabel={t('create.identity.email_verified.switch_off')}
            />
          </FieldRow>
        </Section>
      </div>

      <SaveBar
        show={canSubmit}
        title={t('create.save_bar.title')}
        description={t('create.save_bar.description')}
        onCancel={onBack}
        actions={[{ label: t('create.save_bar.submit'), onClick: onSubmit }]}
      />
    </PageShell>
  )
}
