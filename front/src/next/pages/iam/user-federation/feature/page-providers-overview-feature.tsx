import { useMemo } from 'react'
import { useParams } from 'react-router'
import { useGetUserFederations } from '@/api/user-federation.api'
import { useCreatePicker } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { NEXT_USER_FEDERATION_URL } from '@/next/routes'
import PageProvidersOverview, { type FederationKind } from '../ui/page-providers-overview'

import ProviderResponse = Schemas.ProviderResponse

export default function PageProvidersOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'
  const picker = useCreatePicker()

  const { data: providersResponse, isLoading } = useGetUserFederations(realm)
  const providers = useMemo(() => providersResponse?.data ?? [], [providersResponse])

  const base = NEXT_USER_FEDERATION_URL(realm)

  return (
    <PageProvidersOverview
      providers={providers}
      isLoading={isLoading}
      pickerOpen={picker.open}
      onPickerOpenChange={picker.setOpen}
      createUrl={(kind: FederationKind) => `${base}/create?kind=${kind}`}
      providerHref={(provider: ProviderResponse) => `${base}/${provider.id}/settings`}
    />
  )
}
