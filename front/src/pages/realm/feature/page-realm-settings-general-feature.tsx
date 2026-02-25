import { useForm } from 'react-hook-form'
import { UpdateRealmSchema, updateRealmValidator } from '../validators'
import { zodResolver } from '@hookform/resolvers/zod'
import { SigningAlgorithm } from '@/api/core.interface'
import { Form } from '@/components/ui/form'
import PageRealmSettingsGeneral from '../ui/page-realm-settings-general'
import { useFormChanges } from '@/hooks/use-form-changes'
import { useNavigate, useParams } from 'react-router-dom'
import { RouterParams } from '@/routes/router'
import { useDeleteRealm, useGetRealm } from '@/api/realm.api'
import { toast } from 'sonner'

export default function PageRealmSettingsGeneralFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data: realm } = useGetRealm({ realm: realm_name })
  const navigate = useNavigate()
  const deleteRealm = useDeleteRealm()

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

  const handleDeleteRealm = () => {
    if (!realm_name) return

    deleteRealm.mutate(
      { path: { name: realm_name } },
      {
        onSuccess: () => {
          toast.success(`Realm "${realm_name}" has been deleted.`)
          navigate('/realms/master/overview')
        },
      }
    )
  }

  if (!realm) return (
    <div>No realm</div>
  )

  return (
    <Form {...form}>
      <PageRealmSettingsGeneral
        hasChanges={hasChanges}
        realmName={realm_name ?? ''}
        isMaster={realm_name === 'master'}
        onDeleteRealm={handleDeleteRealm}
      />
    </Form>
  )
}
