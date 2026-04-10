import { FormField } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { UpdatePasswordPolicySchema } from '../validators'
import { useFormContext } from 'react-hook-form'
import { Switch } from '@/components/ui/switch'
import FloatingActionBar from '@/components/ui/floating-action-bar'

export interface PageRealmSettingsPasswordPolicyProps {
  hasChanges: boolean
  onSave: () => void
}

export default function PageRealmSettingsPasswordPolicy({ hasChanges, onSave }: PageRealmSettingsPasswordPolicyProps) {
  const form = useFormContext<UpdatePasswordPolicySchema>()

  return (
    <div className='flex flex-col gap-8'>
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Realm configuration</p>
          <h2 className='text-base font-semibold'>Password Policy</h2>
          <p className='text-sm text-muted-foreground mt-1'>
            Define the rules for user passwords in this realm.
          </p>
        </div>

        <FormField
          control={form.control}
          name='min_length'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Minimum Length</p>
                <p className='text-sm text-muted-foreground mt-0.5'>The minimum number of characters required.</p>
              </div>
              <div className='w-1/2'>
                <InputText
                  type='number'
                  label='Min Length'
                  {...field}
                  onChange={field.onChange}
                  value={field.value ?? ''}
                />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='require_uppercase'
          render={({ field }) => (
            <div className='flex items-center justify-between py-4 border-t'>
              <div className='w-2/3'>
                <p className='text-sm font-medium'>Require Uppercase</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Force users to include at least one uppercase letter.</p>
              </div>
              <Switch checked={!!field.value} onCheckedChange={field.onChange} />
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='require_lowercase'
          render={({ field }) => (
            <div className='flex items-center justify-between py-4 border-t'>
              <div className='w-2/3'>
                <p className='text-sm font-medium'>Require Lowercase</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Force users to include at least one lowercase letter.</p>
              </div>
              <Switch checked={!!field.value} onCheckedChange={field.onChange} />
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='require_number'
          render={({ field }) => (
            <div className='flex items-center justify-between py-4 border-t'>
              <div className='w-2/3'>
                <p className='text-sm font-medium'>Require Number</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Force users to include at least one numeric digit.</p>
              </div>
              <Switch checked={!!field.value} onCheckedChange={field.onChange} />
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='require_special'
          render={({ field }) => (
            <div className='flex items-center justify-between py-4 border-t'>
              <div className='w-2/3'>
                <p className='text-sm font-medium'>Require Special Character</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Force users to include at least one special character (!@#$%...).</p>
              </div>
              <Switch checked={!!field.value} onCheckedChange={field.onChange} />
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='max_age_days'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Password Expiry (Days)</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Force password change after this many days. 0 to disable.</p>
              </div>
              <div className='w-1/2'>
                <InputText
                  type='number'
                  label='Max Age (Days)'
                  {...field}
                  onChange={field.onChange}
                  value={field.value ?? ''}
                />
              </div>
            </div>
          )}
        />
      </div>

      <FloatingActionBar
        show={hasChanges}
        title='Save Changes'
        actions={[{ label: 'Save', variant: 'default', onClick: onSave }]}
        description="You have unsaved changes. Click 'Save' to apply them."
        onCancel={form.reset}
      />
    </div>
  )
}
