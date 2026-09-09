import { useState } from 'react'
import { AlertTriangle } from 'lucide-react'
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Input } from '@/components/ui/input'
import { Button } from './button'
import { cn } from '@/lib/utils'
import { toConfirmToken } from './confirm-token'

export interface ConfirmDestructiveDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  resourceName: string
  title: string
  description: string
  consequences?: string[]
  confirmLabel?: string
  pending?: boolean
  onConfirm: () => void
}

export function ConfirmDestructiveDialog({
  open,
  onOpenChange,
  resourceName,
  title,
  description,
  consequences,
  confirmLabel = 'Delete',
  pending = false,
  onConfirm,
}: ConfirmDestructiveDialogProps) {
  const [draft, setDraft] = useState('')

  const token = toConfirmToken(resourceName)
  const typed = toConfirmToken(draft)
  const matches = token.length > 0 && typed === token
  const mismatch = draft.trim().length > 0 && !matches

  const close = (next: boolean) => {
    if (!next) setDraft('')
    onOpenChange(next)
  }

  const confirm = () => {
    if (!matches || pending) return
    setDraft('')
    onConfirm()
  }

  return (
    <AlertDialog open={open} onOpenChange={close}>
      <AlertDialogContent className='sm:max-w-md'>
        <AlertDialogHeader>
          <div className='flex items-center gap-2.5'>
            <span className='grid size-9 shrink-0 place-items-center rounded-md bg-fk-danger-soft text-fk-danger'>
              <AlertTriangle className='size-4' strokeWidth={1.75} />
            </span>
            <AlertDialogTitle className='text-base'>{title}</AlertDialogTitle>
          </div>
          <AlertDialogDescription className='pt-1'>{description}</AlertDialogDescription>
        </AlertDialogHeader>

        {consequences && consequences.length > 0 && (
          <ul className='space-y-1 rounded-md border border-fk-danger-border bg-fk-danger-soft/40 px-3 py-2.5'>
            {consequences.map((line) => (
              <li key={line} className='flex gap-2 text-xs text-neutral-700 dark:text-neutral-300'>
                <span className='mt-1.5 size-1 shrink-0 rounded-full bg-fk-danger' />
                {line}
              </li>
            ))}
          </ul>
        )}

        <div className='space-y-1.5'>
          <label htmlFor='confirm-token' className='block text-sm text-neutral-700 dark:text-neutral-300'>
            Type <span className='font-mono-ui text-fk-danger'>{token}</span> to confirm.
          </label>
          <Input
            id='confirm-token'
            autoFocus
            value={draft}
            autoComplete='off'
            spellCheck={false}
            aria-invalid={mismatch}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault()
                confirm()
              }
            }}
            placeholder={token}
          />
          <p
            className={cn(
              'text-xs',
              mismatch ? 'text-fk-danger' : 'text-transparent select-none'
            )}
          >
            {mismatch ? 'This does not match the name above.' : '.'}
          </p>
        </div>

        <AlertDialogFooter>
          <Button variant='ghost' size='sm' onClick={() => close(false)}>
            Cancel
          </Button>
          <Button
            size='sm'
            disabled={!matches || pending}
            onClick={confirm}
            className='bg-fk-danger text-white hover:bg-fk-danger/90'
          >
            {confirmLabel}
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
