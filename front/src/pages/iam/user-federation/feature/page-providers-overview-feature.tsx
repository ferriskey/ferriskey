import { useParams } from 'react-router'
import { useCreatePicker } from '@/components/kit'
import type { RouterParams } from '@/routes/router'
import type { Schemas } from '@/api/api.client'
import { USER_FEDERATION_URL } from '@/routes/router'
import PageProvidersOverview, { type FederationKind } from '../ui/page-providers-overview'
import { useProvidersOverview } from './use-providers-overview'

type ProviderResponse = Schemas.ProviderResponse

export default function PageProvidersOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'
  const picker = useCreatePicker()

  const overview = useProvidersOverview(realm)

  const base = USER_FEDERATION_URL(realm)

  return (
    <PageProvidersOverview
      {...overview}
      pickerOpen={picker.open}
      onPickerOpenChange={picker.setOpen}
      createUrl={(kind: FederationKind) => `${base}/create?kind=${kind}`}
      providerHref={(provider: ProviderResponse) => `${base}/${provider.id}/settings`}
    />
  )
}
