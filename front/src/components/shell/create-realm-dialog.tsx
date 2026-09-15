import { useState } from 'react'
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
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

const createRealmSchema = z.object({
  name: z
    .string()
    .trim()
    .min(1, { message: 'A realm name is required' })
    .regex(/^[a-zA-Z0-9_-]+$/, {
      message: 'Only letters, digits, hyphens and underscores are allowed',
    }),
  displayName: z
    .string()
    .trim()
    .max(255, { message: 'Display name must be at most 255 characters' }),
})

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
          toast.success(`Realm ${created.name} created`)
          handleOpenChange(false)
          onCreated(created.name)
        },
        onError: (error) => toast.error(error.message || 'Failed to create realm'),
      }
    )
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create a realm</DialogTitle>
          <DialogDescription>
            A realm owns its own users, credentials, roles and clients. Realms are isolated from one
            another.
          </DialogDescription>
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
              label='Name'
              description='Used in URLs and tokens. It cannot be changed later.'
              htmlFor='create-realm-name'
            >
              <Input
                id='create-realm-name'
                autoFocus
                value={name}
                onChange={(event) => handleNameChange(event.target.value)}
                placeholder='acme'
                aria-invalid={touched && Boolean(nameError)}
              />
              {touched && nameError && (
                <p className='mt-1.5 text-xs text-destructive'>{nameError}</p>
              )}
            </FieldRow>

            <FieldRow
              layout='stacked'
              label='Display name'
              description='Shown to users on the login pages. Follows the name until you edit it.'
              htmlFor='create-realm-display-name'
            >
              <Input
                id='create-realm-display-name'
                value={displayName}
                onChange={(event) => setDisplayName(event.target.value)}
                placeholder='Acme Corporation'
                aria-invalid={touched && Boolean(displayNameError)}
              />
              {touched && displayNameError && (
                <p className='mt-1.5 text-xs text-destructive'>{displayNameError}</p>
              )}
            </FieldRow>
          </div>

          <DialogFooter>
            <Button type='button' variant='ghost' onClick={() => handleOpenChange(false)}>
              Cancel
            </Button>
            <Button type='submit' disabled={isPending}>
              {isPending ? 'Creating…' : 'Create realm'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
