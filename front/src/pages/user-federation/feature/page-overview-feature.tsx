import { useNavigate, useParams } from 'react-router'
import PageOverview from '../ui/page-overview'
import { RouterParams } from '@/routes/router'
import { USER_FEDERATION_URL, USER_FEDERATION_CREATE_URL } from '@/routes/sub-router/user-federation.router'

export default function PageOverviewFeature() {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()

  const handleCreateProvider = () => {
    navigate(`${USER_FEDERATION_URL(realm_name)}${USER_FEDERATION_CREATE_URL}`)
  }

  return (
    <PageOverview onCreateProvider={handleCreateProvider} />
  )
}
