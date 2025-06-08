import { useFormContext } from 'react-hook-form';
import { FormField } from '@/components/ui/form.tsx';
import { FormSwitch } from '@/components/ui/switch.tsx';
import { UpdateUserSchema } from '../validators';
import BlockContent from '@/components/ui/block-content';
import { InputText } from '@/components/ui/input-text';
import FloatingActionBar from '@/components/ui/floating-action-bar.tsx'

type Props = {
  onSubmit: (data: UpdateUserSchema) => void
  hasChanges: boolean
}

export default function PageUserOverview({ onSubmit, hasChanges }: Props) {
  const form = useFormContext<UpdateUserSchema>()

  return (
    <div className="max-w-2xl">
      <BlockContent title='General information'>
        <div className='flex flex-col gap-4'>
          <FormField 
            control={form.control}
            name="username"
            render={({ field }) => (
              <InputText 
                label='Username'
                disabled
                {...field} 
              />
            )}
          />

          <FormField
            control={form.control}
            name="email"
            render={({ field }) => (
              <InputText
                label='Email'
                type='email'
                {...field}
              />
            )}
          />

          <FormField
            control={form.control}
            name="firstname"
            render={({ field }) => (
              <InputText
                label='First Name'
                {...field}
              />
            )}
          />

          <FormField
            control={form.control}
            name="lastname"
            render={({ field }) => (
              <InputText
                label='Last Name'
                {...field}
              />
            )}
          />

          <FormField
            control={form.control}
            name="enabled"
            render={({ field }) => (
              <FormSwitch
                label="User Enabled"
                description="Choose between enabled and disabled user as default status."
                checked={field.value}
                onChange={field.onChange}
              />
            )}
          />

          <FormField
            control={form.control}
            name="email_verified"
            render={({ field }) => (
              <FormSwitch
                label="Email Verified"
                description={(value) => value ? "Email is verified" : "Email is not verified"}
                checked={field.value}
                onChange={field.onChange}
              />
            )}
          />

        </div>

      </BlockContent>

      <FloatingActionBar
        show={hasChanges}
        title={"Save changes"}
        actions={[
          {
            label: 'Save',
            variant: 'default',
            onClick: form.handleSubmit(onSubmit)
          }
        ]}
        description="You have unsaved changes. Click 'Save' to apply them."
        onCancel={() => {
            form.reset()
        }}
      />
    </div>
  )
}
