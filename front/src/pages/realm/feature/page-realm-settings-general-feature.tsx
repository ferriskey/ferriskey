import { useForm } from 'react-hook-form'
import { UpdateRealmSchema, updateRealmValidator } from '../validators'
import { zodResolver } from '@hookform/resolvers/zod'
import { SigningAlgorithm } from '@/api/core.interface'
import { Form } from '@/components/ui/form'
import PageRealmSettingsGeneral from '../ui/page-realm-settings-general'
import { useFormChanges } from '@/hooks/use-form-changes'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useGetRealm } from '@/api/realm.api'

export default function PageRealmSettingsGeneralFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data: realm } = useGetRealm({ realm: realm_name })

  const form = useForm<UpdateRealmSchema>({
    resolver: zodResolver(updateRealmValidator),
    mode: 'all',
    values: {
      name: realm?.name ?? 'master',
      default_signing_algorithm: SigningAlgorithm.RS256,
    }
  })

  const hasChanges = useFormChanges(
    form,
    realm && {
      name: realm.name ?? 'master',
      default_signing_algorithm: SigningAlgorithm.RS256,
    }
  )


  if (!realm) return (
    <div>No realm</div>
  )

  return (
    <Form {...form}>
      <PageRealmSettingsGeneral hasChanges={hasChanges} />
    </Form>
  )
}
