import { useParams } from 'react-router'
import { toast } from 'sonner'
import { useGetRealmPasswordPolicy, useUpdateRealmPasswordPolicy } from '@/api/realm.api'
import { updatePasswordPolicyValidator } from '@/pages/iam/realm/validators'
import { RouterParams } from '@/routes/router'
import { useDraft } from '@/pages/iam/realm/feature/use-draft'
import type { PolicyDraft } from '@/pages/iam/realm/ui/realm-password-policy-tab'
import PagePasswordPolicy from '../ui/page-password-policy'

const DEFAULT_POLICY: PolicyDraft = {
  min_length: 8,
  require_uppercase: false,
  require_lowercase: false,
  require_number: false,
  require_special: false,
  max_age_days: 0,
  min_entropy_bits: 0,
  forbid_common: false,
  check_breached: false,
}

export default function PagePasswordPolicyFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { data: policyData, isLoading, isError } = useGetRealmPasswordPolicy({ realm })
  const { mutate: updatePolicy, isPending } = useUpdateRealmPasswordPolicy()

  const pristine: PolicyDraft = policyData
    ? {
        min_length: policyData.min_length,
        require_uppercase: policyData.require_uppercase,
        require_lowercase: policyData.require_lowercase,
        require_number: policyData.require_number,
        require_special: policyData.require_special,
        max_age_days: policyData.max_age_days ?? 0,
        min_entropy_bits: policyData.min_entropy_bits,
        forbid_common: policyData.forbid_common,
        check_breached: policyData.check_breached,
      }
    : DEFAULT_POLICY

  const draft = useDraft(
    policyData ? `${policyData.id}:${policyData.updated_at}` : '',
    pristine
  )
  const value = draft.value

  const parsed = updatePasswordPolicyValidator.safeParse(value)
  const errors: Partial<Record<keyof PolicyDraft, string>> = {}
  if (!parsed.success) {
    for (const issue of parsed.error.issues) {
      const field = issue.path[0] as keyof PolicyDraft
      if (!errors[field]) errors[field] = issue.message
    }
  }

  const changed = (Object.keys(pristine) as (keyof PolicyDraft)[]).filter(
    (key) => value[key] !== pristine[key]
  )

  const save = () => {
    if (!realm_name || !parsed.success || changed.length === 0 || isPending) return

    updatePolicy(
      { path: { realm_name }, body: value },
      {
        onSuccess: () => toast.success('Password policy updated.'),
        onError: (error: Error) =>
          toast.error(error.message || 'Failed to update the password policy'),
      }
    )
  }

  return (
    <PagePasswordPolicy
      value={value}
      errors={errors}
      isLoading={isLoading}
      failed={!isLoading && (isError || !policyData)}
      dirtyCount={changed.length}
      canSave={parsed.success}
      isSaving={isPending}
      onChange={draft.patch}
      onDiscard={draft.reset}
      onSave={save}
    />
  )
}
