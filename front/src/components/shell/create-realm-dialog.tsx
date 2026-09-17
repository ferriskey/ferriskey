import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { z } from 'zod'
import { useCreateRealm } from '@/api/realm.api'
import { Button } from '@/components/kit/button'
import { FieldRow } from '@/components/kit/form'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { translate } from '@/lib/i18n'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

const createRealmSchema = z.object({
  name: z
    .string()
    .trim()
    .min(1, { error: () => translate('common:realm_dialog.validation.name_required') })
    .regex(/^[a-zA-Z0-9_-]+$/, {
      error: () => translate('common:realm_dialog.validation.name_charset'),
    }),
  displayName: z
    .string()
    .trim()
    .max(255, { error: () => translate('common:realm_dialog.validation.display_name_length') }),
})

const REALM_NAME_EXAMPLE = 'acme'
const REALM_DISPLAY_NAME_EXAMPLE = 'Acme Corporation'

export interface CreateRealmDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  realm: string
  onCreated: (realmName: string) => void
}

export function CreateRealmDialog({
  open,
  onOpenChange,
  realm,
  onCreated,
}: CreateRealmDialogProps) {
  const { t } = useTranslation()
  const { mutate: createRealm, isPending } = useCreateRealm({ realm })
  const [name, setName] = useState('')
  const [displayName, setDisplayName] = useState('')
  const [touched, setTouched] = useState(false)

  const handleNameChange = (next: string) => {
    const hyphenated = next.replace(/\s+/g, '-')
    setName(hyphenated)
    setDisplayName(hyphenated.replace(/-/g, ' '))
  }

  const handleOpenChange = (next: boolean) => {
    if (!next) {
      setName('')
      setDisplayName('')
      setTouched(false)
    }
    onOpenChange(next)
  }

  const parsed = createRealmSchema.safeParse({ name, displayName })
  const nameError = parsed.success
    ? undefined
    : parsed.error.issues.find((issue) => issue.path[0] === 'name')?.message
  const displayNameError = parsed.success
    ? undefined
    : parsed.error.issues.find((issue) => issue.path[0] === 'displayName')?.message

  const handleSubmit = () => {
    setTouched(true)
    if (!parsed.success) return

    createRealm(
      {
        body: {
          name: parsed.data.name,
          display_name: parsed.data.displayName || null,
        },
      },
      {
        onSuccess: (created) => {
          toast.success(t('realm_dialog.toast.created', { name: created.name }))
          handleOpenChange(false)
          onCreated(created.name)
        },
        onError: (error) =>
          toast.error(error.message || t('realm_dialog.toast.create_failed')),
      }
    )
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('realm_dialog.title')}</DialogTitle>
          <DialogDescription>{t('realm_dialog.description')}</DialogDescription>
        </DialogHeader>

        <form
          onSubmit={(event) => {
            event.preventDefault()
            handleSubmit()
          }}
        >
          <div className={cn(tokens.surface.panel, 'px-4', tokens.surface.divider)}>
            <FieldRow
              layout='stacked'
              label={t('realm_dialog.name.label')}
              description={t('realm_dialog.name.description')}
              htmlFor='create-realm-name'
            >
              <Input
                id='create-realm-name'
                autoFocus
                value={name}
                onChange={(event) => handleNameChange(event.target.value)}
                placeholder={REALM_NAME_EXAMPLE}
                aria-invalid={touched && Boolean(nameError)}
              />
              {touched && nameError && (
                <p className='mt-1.5 text-xs text-destructive'>{nameError}</p>
              )}
            </FieldRow>

            <FieldRow
              layout='stacked'
              label={t('realm_dialog.display_name.label')}
              description={t('realm_dialog.display_name.description')}
              htmlFor='create-realm-display-name'
            >
              <Input
                id='create-realm-display-name'
                value={displayName}
                onChange={(event) => setDisplayName(event.target.value)}
                placeholder={REALM_DISPLAY_NAME_EXAMPLE}
                aria-invalid={touched && Boolean(displayNameError)}
              />
              {touched && displayNameError && (
                <p className='mt-1.5 text-xs text-destructive'>{displayNameError}</p>
              )}
            </FieldRow>
          </div>

          <DialogFooter>
            <Button type='button' variant='ghost' onClick={() => handleOpenChange(false)}>
              {t('action.cancel')}
            </Button>
            <Button type='submit' disabled={isPending}>
              {isPending ? t('realm_dialog.submit_pending') : t('realm_dialog.submit')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
