import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { ChoiceCards, type Choice } from './form'

export function CreatePickerDialog<T extends string>({
  open,
  onOpenChange,
  title,
  description,
  options,
  defaultValue,
  createUrl,
  confirmLabel = 'Continue',
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  description: string
  options: Choice<T>[]
  defaultValue: T
  createUrl: (value: T) => string
  confirmLabel?: string
}) {
  const navigate = useNavigate()
  const [value, setValue] = useState<T>(defaultValue)

  const handleOpenChange = (next: boolean) => {
    if (!next) setValue(defaultValue)
    onOpenChange(next)
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className='sm:max-w-xl'>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>

        <ChoiceCards
          label={title}
          value={value}
          onChange={setValue}
          options={options}
          className='grid-cols-2 max-w-none'
        />

        <DialogFooter>
          <Button variant='ghost' onClick={() => handleOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={() => navigate(createUrl(value))}>{confirmLabel}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
