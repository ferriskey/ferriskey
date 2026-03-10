import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'

interface DangerZoneProps {
  label: string
  description: string
  buttonLabel: string
  confirmTitle: string
  confirmDescription: string
  confirmText?: string
  disabled?: boolean
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
  onConfirm,
}: DangerZoneProps) {
  const [open, setOpen] = useState(false)

  return (
    <>
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-destructive/70 mb-0.5'>Irreversible actions</p>
          <h2 className='text-base font-semibold text-destructive'>Danger Zone</h2>
        </div>

        <div className='flex items-center justify-between py-4 border-t border-destructive/20'>
          <div className='w-2/3'>
            <p className='text-sm font-medium'>{label}</p>
            <p className='text-sm text-muted-foreground mt-0.5'>{description}</p>
          </div>
          <Button variant='destructive' disabled={disabled} onClick={() => setOpen(true)}>
            {buttonLabel}
          </Button>
        </div>
      </div>

      <ConfirmDeleteAlert
        open={open}
        title={confirmTitle}
        description={confirmDescription}
        confirmText={confirmText}
        onConfirm={onConfirm}
        onCancel={() => setOpen(false)}
      />
    </>
  )
}
