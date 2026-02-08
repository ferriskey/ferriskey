import { useState } from 'react'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Input } from '@/components/ui/input'
import { Trash2 } from 'lucide-react'

interface ConfirmDeleteAlertProps {
  open: boolean
  title: string
  description: string
  onConfirm: () => void
  onCancel: () => void
  confirmText?: string
}

export function ConfirmDeleteAlert({
  open,
  title,
  description,
  onConfirm,
  onCancel,
  confirmText,
}: ConfirmDeleteAlertProps) {
  const [inputValue, setInputValue] = useState('')

  const isConfirmDisabled = confirmText ? inputValue !== confirmText : false

  const handleCancel = () => {
    setInputValue('')
    onCancel()
  }

  return (
    <AlertDialog open={open} onOpenChange={(isOpen) => !isOpen && handleCancel()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <div className='flex gap-2 items-center'>
            <div className='bg-primary/10 text-primary p-2 rounded-full'>
              <Trash2 className='h-5 w-5' />
            </div>
            <AlertDialogTitle>{title}</AlertDialogTitle>
          </div>
          <AlertDialogDescription>{description}</AlertDialogDescription>
        </AlertDialogHeader>
        {confirmText && (
          <div className='flex flex-col gap-2'>
            <p className='text-sm text-muted-foreground'>
              Type <span className='font-semibold'>{confirmText}</span> to confirm.
            </p>
            <Input
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              placeholder={confirmText}
            />
          </div>
        )}
        <AlertDialogFooter>
          <AlertDialogCancel className='outline-none cursor-pointer'>
            Cancel
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={onConfirm}
            disabled={isConfirmDisabled}
            className='bg-destructive text-destructive-foreground hover:bg-destructive/90 px-4 py-1 rounded-md text-white ml-3 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed'
          >
            Confirm
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
