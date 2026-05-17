import { useGetClients } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { APPLICATION_CREATE_URL, APPLICATIONS_URL } from '@/routes/sub-router/applications.router'
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
    // Detail page lives in admin for now; we'll add a CIAM detail later.
    navigate(`${APPLICATIONS_URL(realm_name)}`)
    void clientId
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
