import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { FormControl, FormDescription, FormField, FormItem, FormLabel } from '@/components/ui/form'
import { Dispatch, SetStateAction } from 'react'
import { useFormContext } from 'react-hook-form'
import { Switch } from '../../../components/ui/switch'
import { CreateUserSchema } from '../validators'
import { InputText } from '@/components/ui/input-text'

type Props = {
  realm: string
  onSubmit: (data: CreateUserSchema) => void
  openState: [boolean, Dispatch<SetStateAction<boolean>>]
}

export default function CreateUserModal(props: Props) {
  const form = useFormContext<CreateUserSchema>()
  const [open, setOpen] = props.openState

  const onOpenChange = (open: boolean) => {
    setOpen(open)
    form.reset()
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className='sm:max-w-[425px]'>
        <DialogHeader>
          <DialogTitle>Create User</DialogTitle>
          <DialogDescription>
            Create a new user in the selected realm: {props.realm}.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={form.handleSubmit(props.onSubmit)}>
          <div className='grid gap-5 py-4'>
            <div className='flex flex-col gap-1'>
              <FormField
                control={form.control}
                name='username'
                render={({ field, formState }) => (
                  <InputText
                    label='Username'
                    name='username'
                    value={field.value}
                    onChange={field.onChange}
                    error={formState.errors.username?.message}
                  />
                )}
              />
            </div>

            <div className='flex flex-col gap-1'>
              <FormField
                control={form.control}
                name='firstname'
                render={({ field, formState }) => (
                  <InputText
                    label='First Name'
                    name='firstname'
                    value={field.value}
                    onChange={field.onChange}
                    error={formState.errors.firstname?.message}
                  />
                )}
              />
            </div>

            <div className='flex flex-col gap-1'>
              <FormField
                control={form.control}
                name='lastname'
                render={({ field, formState }) => (
                  <InputText
                    label='Last Name'
                    name='lastname'
                    value={field.value}
                    onChange={field.onChange}
                    error={formState.errors.lastname?.message}
                  />
                )}
              />
            </div>

            <div className='flex flex-col gap-1'>
              <FormField
                control={form.control}
                name='email'
                render={({ field, formState }) => (
                  <InputText
                    label='Email'
                    name='email'
                    value={field.value}
                    onChange={field.onChange}
                    error={formState.errors.email?.message}
                  />
                )}
              />
            </div>

            <div className='flex flex-col gap-1'>
              <FormField
                control={form.control}
                name='email_verified'
                render={({ field }) => (
                  <FormItem className='flex flex-row items-center justify-between gap-5 rounded-lg border p-3 shadow-sm'>
                    <div className='space-y-0.5'>
                      <FormLabel>Verified email</FormLabel>
                      <FormDescription>
                        Choose between verified and unverified email as default status.
                      </FormDescription>
                    </div>
                    <FormControl>
                      <Switch checked={field.value} onCheckedChange={field.onChange} />
                    </FormControl>
                  </FormItem>
                )}
              />
            </div>
          </div>
          <DialogFooter>
            <Button type='submit'>Save changes</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
