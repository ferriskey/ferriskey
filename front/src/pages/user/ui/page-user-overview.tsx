import { useFormContext } from 'react-hook-form'
import { FormField } from '@/components/ui/form.tsx'
import { FormSwitch } from '@/components/ui/switch.tsx'
import { UpdateUserSchema } from '../validators'
import BlockContent from '@/components/ui/block-content'
import { InputText } from '@/components/ui/input-text'
import FloatingActionBar from '@/components/ui/floating-action-bar.tsx'
import { RequiredAction, User } from '@/api/core.interface'
import MultipleSelector from '@/components/ui/multiselect'
import { Label } from '@/components/ui/label'
import { formatRequiredAction, formatSnakeCaseToTitleCase } from '@/utils'

type Props = {
  onSubmit: (data: UpdateUserSchema) => void
  hasChanges: boolean
  user: User
}

export default function PageUserOverview({ onSubmit, hasChanges, user }: Props) {
  const form = useFormContext<UpdateUserSchema>()

  const requiredActions = Object.values(RequiredAction).map((action) => {
    return {
      label: formatRequiredAction(action),
      value: action,
    }
  })

  return (
    <div className="max-w-2xl">
      <BlockContent title="User details">
        <div className="flex flex-col gap-3">
          <InputText label="User ID" value={user.id} disabled={true} name="id" />

          <InputText
            label="Created At"
            value={new Date(user.created_at).toLocaleDateString('en-US', {
              year: 'numeric',
              month: '2-digit',
              day: '2-digit',
            })}
            disabled={true}
            name="created_at"
          />

          <div>
            <FormField
              control={form.control}
              name="required_actions"
              render={({ field }) => (
                <div>
                  <Label>Required Actions</Label>
                  <MultipleSelector
                    commandProps={{
                      label: 'Required Actions',
                    }}
                    onChange={(value) => field.onChange(value.map((v) => v.value))}
                    value={field.value?.map((action) => ({
                      label: formatSnakeCaseToTitleCase(action),
                      value: action,
                    }))}
                    options={requiredActions}
                  />
                </div>
              )}
            />
          </div>
        </div>
      </BlockContent>
      <BlockContent title="General information">
        <div className="flex flex-col gap-4">
          <FormField
            control={form.control}
            name="username"
            render={({ field }) => <InputText label="Username" disabled {...field} />}
          />

          <FormField
            control={form.control}
            name="email"
            render={({ field }) => <InputText label="Email" type="email" {...field} />}
          />

          <FormField
            control={form.control}
            name="firstname"
            render={({ field }) => <InputText label="First Name" {...field} />}
          />

          <FormField
            control={form.control}
            name="lastname"
            render={({ field }) => <InputText label="Last Name" {...field} />}
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
                description={(value) => (value ? 'Email is verified' : 'Email is not verified')}
                checked={field.value}
                onChange={field.onChange}
              />
            )}
          />
        </div>
      </BlockContent>

      <FloatingActionBar
        show={hasChanges}
        title={'Save changes'}
        actions={[
          {
            label: 'Save',
            variant: 'default',
            onClick: form.handleSubmit(onSubmit),
          },
        ]}
        description="You have unsaved changes. Click 'Save' to apply them."
        onCancel={() => {
          form.reset()
        }}
      />
    </div>
  )
}
