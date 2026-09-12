import { useState } from 'react'
import { Button } from '@/components/kit/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { ConfirmDestructiveDialog } from './confirm-destructive-dialog'

export interface DangerZoneProps {
  label: string
  description: string
  buttonLabel: string
  confirmTitle: string
  confirmDescription: string
  confirmText?: string
  disabled?: boolean
  disabledReason?: string
  /**
   * Name of the resource. Given, the confirmation asks the administrator to
   * retype it before the action is allowed.
   */
  resourceName?: string
  consequences?: string[]
  pending?: boolean
  onConfirm: () => void
}

export function DangerZone({
  label,
  description,
  buttonLabel,
  confirmTitle,
  confirmDescription,
  confirmText,
  disabled,
  disabledReason,
  resourceName,
  consequences,
  pending,
  onConfirm,
}: DangerZoneProps) {
  const [open, setOpen] = useState(false)

  return (
    <>
      <section className='rounded-lg border border-fk-danger-border bg-fk-danger-soft/40 p-5'>
        <div className='flex flex-wrap items-start justify-between gap-4'>
          <div className='min-w-0 max-w-xl'>
            <h3 className='text-sm font-semibold text-fk-danger'>{label}</h3>
            <p className='mt-1 text-sm text-neutral-600 dark:text-neutral-400'>{description}</p>
            {disabled && disabledReason && (
              <p className='mt-2 text-xs text-fk-danger'>{disabledReason}</p>
            )}
          </div>
          <Button
            size='lg'
            disabled={disabled}
            title={disabled ? disabledReason : undefined}
            onClick={() => setOpen(true)}
            className='bg-fk-danger text-white hover:bg-fk-danger/90'
          >
            {buttonLabel}
          </Button>
        </div>
      </section>

      {resourceName ? (
        <ConfirmDestructiveDialog
          open={open}
          onOpenChange={setOpen}
          resourceName={resourceName}
          title={confirmTitle}
          description={confirmDescription}
          consequences={consequences}
          confirmLabel={buttonLabel}
          pending={pending}
          onConfirm={onConfirm}
        />
      ) : (
        <ConfirmDeleteAlert
          open={open}
          title={confirmTitle}
          description={confirmDescription}
          confirmText={confirmText}
          onConfirm={onConfirm}
          onCancel={() => setOpen(false)}
        />
      )}
    </>
  )
}
