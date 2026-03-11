import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import PageRealmSettingsPasswordPolicy from '../ui/page-realm-settings-password-policy'
import { useGetPasswordPolicy, useUpdatePasswordPolicy } from '@/api/realm.api'
import { useEffect } from 'react'
import { useFormChanges } from '@/hooks/use-form-changes'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'

const passwordPolicySchema = z.object({
    minLength: z.number().min(1).max(128),
    requireUppercase: z.boolean(),
    requireLowercase: z.boolean(),
    requireNumber: z.boolean(),
    requireSpecial: z.boolean(),
    maxAgeDays: z.number().nullable(),
})

export type PasswordPolicySchema = z.infer<typeof passwordPolicySchema>

export default function PageRealmSettingsPasswordPolicyFeature() {
    const { realm_name } = useParams<RouterParams>()
    const { data } = useGetPasswordPolicy({ realm: realm_name })
    const { mutate } = useUpdatePasswordPolicy()

    const form = useForm<PasswordPolicySchema>({
        resolver: zodResolver(passwordPolicySchema),
        defaultValues: {
            minLength: 8,
            requireUppercase: false,
            requireLowercase: false,
            requireNumber: false,
            requireSpecial: false,
            maxAgeDays: null,
        }
    })

    const handleSubmit = (values: PasswordPolicySchema) => {
        if (!realm_name) return

        mutate({
            path: {
                realm_name: realm_name
            },
            body: {
                min_length: values.minLength,
                require_uppercase: values.requireUppercase,
                require_lowercase: values.requireLowercase,
                require_number: values.requireNumber,
                require_special: values.requireSpecial,
                max_age_days: values.maxAgeDays,
            }
        })
    }

    const hasChanges = useFormChanges(
        form,
        data && {
            minLength: data.min_length,
            requireUppercase: data.require_uppercase,
            requireLowercase: data.require_lowercase,
            requireNumber: data.require_number,
            require_special: data.require_special,
            maxAgeDays: data.max_age_days,
        }
    )

    useEffect(() => {
        if (data) {
            form.reset({
                minLength: data.min_length,
                requireUppercase: data.require_uppercase,
                requireLowercase: data.require_lowercase,
                requireNumber: data.require_number,
                requireSpecial: data.require_special,
                maxAgeDays: data.max_age_days,
            })
        }
    }, [data, form])

    return (
        <PageRealmSettingsPasswordPolicy form={form} hasChanges={hasChanges} handleSubmit={handleSubmit} />
    )
}
