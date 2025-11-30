import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { InputText } from '@/components/ui/input-text'
import { UseFormReturn } from 'react-hook-form'
import { SetCredentialPasswordSchema } from '@/pages/user/schemas'
import { FormField } from '@/components/ui/form'
import { FormSwitch } from '@/components/ui/switch'
import { Dispatch, SetStateAction } from 'react'


export interface SetPasswordProps {
  form: UseFormReturn<SetCredentialPasswordSchema>
  open: boolean
  setOpen: Dispatch<SetStateAction<boolean>>
  handleCloseModal: () => void
  handleSubmit: () => void
  contentText?: string
}

export default function SetPassword({
  form,
  open,
  setOpen,
  handleCloseModal,
  handleSubmit,
  contentText,
}: SetPasswordProps) {
  const onSubmit = form.handleSubmit(handleSubmit)

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant='outline'>{contentText || 'Set Password'}</Button>
      </DialogTrigger>

      <DialogContent>
        <DialogTitle>Set Password</DialogTitle>
        <DialogDescription>Please enter your new password below.</DialogDescription>
          <div className='flex flex-col gap-4'>
            <FormField
              control={form.control}
              name='password'
              render={({ field, fieldState }) => (
                <InputText
                  label='Password'
                  type='password'
                  error={fieldState.error?.message}
                  {...field}
                />
              )}
            />

            <FormField
              control={form.control}
              name='confirmPassword'
              render={({ field, fieldState }) => (
                <InputText
                  label='Confirm Password'
                  type='password'
                  error={fieldState.error?.message}
                  {...field}
                />
              )}
            />

            <FormField
              control={form.control}
              name='temporary'
              render={({ field }) => (
                <FormSwitch
                  label='Temporary'
                  description='This password is temporary and will require the user to change it on next login.'
                  checked={field.value}
                  onChange={field.onChange}
                />
              )}
            />

            <div className='flex items-center justify-end gap-2 mt-4'>
              <Button variant='ghost' onClick={handleCloseModal}>
                Cancel
              </Button>

              <Button onClick={onSubmit}>Apply</Button>
            </div>
          </div>
      </DialogContent>
    </Dialog>
  )
}
