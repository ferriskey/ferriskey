import { useGetClient } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import PageClientCredentials from '../ui/page-client-credentials'

export default function PageClientCredentialsFeature() {
  const { realm_name, client_id } = useParams<RouterParams>()

  const { data: responseData } = useGetClient({
    realm: realm_name ?? 'master',
    clientId: client_id ?? '',
  })

  if (!responseData) {
    return <div>Loading...</div>
  }

  return (
    <PageClientCredentials client={responseData.data} />
  )
}
