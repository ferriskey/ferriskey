import { useGetClients } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { APPLICATION_CREATE_URL, APPLICATION_DETAIL_URL } from '@/routes/sub-router/applications.router'
import { useNavigate, useParams } from 'react-router'
import PageApplicationsList from '../ui/page-applications-list'

export default function PageApplicationsListFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data, isLoading } = useGetClients({ realm: realm_name ?? 'master' })

  const handleCreate = () => {
    if (!realm_name) return
    navigate(APPLICATION_CREATE_URL(realm_name))
  }

  const handleSelect = (clientId: string) => {
    if (!realm_name) return
    navigate(APPLICATION_DETAIL_URL(realm_name, clientId))
  }

  return (
    <PageApplicationsList
      applications={data?.data ?? []}
      isLoading={isLoading}
      onCreate={handleCreate}
      onSelect={handleSelect}
    />
  )
}
