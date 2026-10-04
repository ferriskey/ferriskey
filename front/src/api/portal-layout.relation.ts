import { useParams } from 'react-router'
import type { RelationOption, RelationSource } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import type { Schemas } from './api.client'
import {
  PORTAL_LAYOUT_SEARCH_LIMIT,
  useGetPortalLayout,
  useGetPortalLayouts,
} from './portal-layouts.api'

const toOption = (layout: Schemas.PortalLayout): RelationOption => ({
  id: layout.id,
  label: layout.name,
})

function useRealm() {
  const { realm_name } = useParams<RouterParams>()
  return realm_name ?? 'master'
}

function useLayoutOptions(search: string) {
  const realm = useRealm()
  const { data, isLoading } = useGetPortalLayouts({
    realm,
    query: {
      name: search.trim() || undefined,
      order_by: 'name',
      order: 'asc',
      limit: PORTAL_LAYOUT_SEARCH_LIMIT,
    },
  })
  return { options: (data?.data ?? []).map(toOption), loading: isLoading }
}

function useSelectedLayout(id: string | undefined) {
  const realm = useRealm()
  const { data } = useGetPortalLayout({ realm, layoutId: id ?? '' })
  return data?.data ? toOption(data.data) : undefined
}

export const portalLayoutRelationSource: RelationSource = {
  useOptions: useLayoutOptions,
  useSelected: useSelectedLayout,
}
