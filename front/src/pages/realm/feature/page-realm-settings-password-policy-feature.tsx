import { useForm } from 'react-hook-form'
import { UpdatePasswordPolicySchema, updatePasswordPolicyValidator } from '../validators'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form } from '@/components/ui/form'
import { useFormChanges } from '@/hooks/use-form-changes'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useGetRealmPasswordPolicy, useUpdateRealmPasswordPolicy } from '@/api/realm.api'
import { toast } from 'sonner'
import PageRealmSettingsPasswordPolicy from '../ui/page-realm-settings-password-policy'

export default function PageRealmSettingsPasswordPolicyFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data: policy, isLoading } = useGetRealmPasswordPolicy({ realm: realm_name })
  const updatePolicy = useUpdateRealmPasswordPolicy()

  const form = useForm<UpdatePasswordPolicySchema>({
    resolver: zodResolver(updatePasswordPolicyValidator),
    mode: 'all',
    values: {
      min_length: policy?.min_length ?? 8,
      require_uppercase: policy?.require_uppercase ?? false,
      require_lowercase: policy?.require_lowercase ?? false,
      require_number: policy?.require_number ?? false,
      require_special: policy?.require_special ?? false,
      max_age_days: policy?.max_age_days ?? 0,
    }
  })

  const hasChanges = useFormChanges(
    form,
    policy && {
      min_length: policy.min_length,
      require_uppercase: policy.require_uppercase,
      require_lowercase: policy.require_lowercase,
      require_number: policy.require_number,
      require_special: policy.require_special,
      max_age_days: policy.max_age_days ?? 0,
    }
  )

  const handleSave = () => {
    if (!realm_name) return

    const values = form.getValues()
    updatePolicy.mutate(
      {
        path: { realm_name },
        body: values,
      },
      {
        onSuccess: () => {
          toast.success('Password policy updated successfully')
        },
        onError: (error: Error) => {
          toast.error(error.message || 'Failed to update password policy')
        }
      }
    )
  }

  if (isLoading) {
    return <div className='p-8'>Loading password policy...</div>
  }

  if (!policy) {
    return <div className='p-8 text-muted-foreground text-sm'>Failed to load password policy.</div>
  }

  return (
    <Form {...form}>
      <PageRealmSettingsPasswordPolicy
        hasChanges={hasChanges}
        onSave={handleSave}
      />
    </Form>
  )
}
