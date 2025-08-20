import { useForm } from 'react-hook-form'
import { UpdateRealmSchema, updateRealmValidator } from '../validators'
import { zodResolver } from '@hookform/resolvers/zod'
import { SigningAlgorithm } from '@/api/core.interface'
import { Form } from '@/components/ui/form'
import PageRealmSettingsGeneral from '../ui/page-realm-settings-general'
import useRealmStore from '@/store/realm.store'
import { mapRealms } from '@/api/core.mapper'
import { useFormChanges } from '@/hooks/use-form-changes'
import { useFeature } from '@/hooks/use-feature'
import { Feature } from '@/lib/features'

export default function PageRealmSettingsGeneralFeature() {
  const enabled = useFeature(Feature.REALM_SETTINGS)
  const { userRealms } = useRealmStore()

  const realm = mapRealms(userRealms).find((item) => item.name === 'master')

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
      default_signing_algorithm:SigningAlgorithm.RS256,
    }
  )


  if (!realm) return null

  if (!enabled) return null

  return (
    <Form {...form}>
      <PageRealmSettingsGeneral realm={realm} hasChanges={hasChanges} />
    </Form>
  )
}
