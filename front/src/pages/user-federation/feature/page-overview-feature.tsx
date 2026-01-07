import { useNavigate, useParams } from 'react-router'
import PageOverview from '../ui/page-overview'
import { RouterParams } from '@/routes/router'
import { USER_FEDERATION_URL, USER_FEDERATION_CREATE_URL } from '@/routes/sub-router/user-federation.router'

export default function PageOverviewFeature() {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()

  const handleCreateProvider = (type?: 'LDAP' | 'Kerberos') => {
    const url = `${USER_FEDERATION_URL(realm_name)}/${type ? type.toLowerCase() : ''}${USER_FEDERATION_CREATE_URL}/`
    navigate(url)
  }

  return (
    <PageOverview onCreateProvider={handleCreateProvider} />
  )
}
