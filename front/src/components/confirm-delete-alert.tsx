import { useEffect, useRef } from 'react'
import { Button } from '@/components/ui/button'
import { Trash2 } from 'lucide-react'

interface ConfirmDeleteAlertProps {
  open: boolean
  title: string
  description: string
  onConfirm: () => void
  onCancel: () => void
}

export function ConfirmDeleteAlert({
  open,
  title,
  description,
  onConfirm,
  onCancel,
}: ConfirmDeleteAlertProps) {
  const overlayRef = useRef<HTMLDivElement>(null)

  // Close on Escape
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onCancel()
    }
    if (open) window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [open, onCancel])

  if (!open) return null

  return (
    <div
      ref={overlayRef}
      className='fixed inset-0 z-50 flex items-center justify-center bg-black/50'
      onClick={onCancel}
    >
      <div
        className='bg-white rounded-lg shadow-lg max-w-md w-full p-6'
        onClick={(e) => e.stopPropagation()}
      >
        <div className='flex gap-2 items-center'>
          <div className='bg-primary/10 text-primary p-2 rounded-full'>
            <Trash2 className='h-5 w-5' />
          </div>
          <p className='font-medium'>{title}</p>
        </div>
        <p className='mt-2 text-sm'>{description}</p>

        <div className='mt-4 flex justify-end gap-2'>
          <Button onClick={onCancel} variant='ghost'>
            Cancel
          </Button>
          <Button onClick={onConfirm} variant='destructive' size='sm'>
            Confirm
          </Button>
        </div>
      </div>
    </div>
  )
}
