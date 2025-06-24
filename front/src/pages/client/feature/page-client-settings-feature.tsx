import { useGetClient } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import PageClientSettings from '../ui/page-client-settings'

export default function PageClientSettingsFeature() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const { data } = useGetClient({
    realm: realm_name ?? 'master',
    clientId: client_id ?? '',
  })

  return <>{data && <PageClientSettings client={data} />}</>
}
