import { useParams } from 'react-router'
import type { RelationOption, RelationSource } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import { GROUP_SEARCH_LIMIT, useGroup, useGroups, type Group } from './group.api'

const toOption = (group: Group): RelationOption => ({
  id: group.id,
  label: group.name,
  sublabel: group.description ?? undefined,
})

function useOrganization() {
  const { realm_name, organizationId } = useParams<RouterParams & { organizationId: string }>()
  return { realm: realm_name, orgId: organizationId }
}

function useGroupOptions(search: string) {
  const { realm, orgId } = useOrganization()
  const { data, isLoading } = useGroups({
    realm,
    orgId,
    query: {
      name: search.trim() || undefined,
      order_by: 'name',
      order: 'asc',
      limit: GROUP_SEARCH_LIMIT,
    },
  })
  return { options: (data?.data ?? []).map(toOption), loading: isLoading }
}

function useSelectedGroup(id: string | undefined) {
  const { realm, orgId } = useOrganization()
  const { data } = useGroup({ realm, orgId, groupId: id })
  return data ? toOption(data) : undefined
}

export const groupRelationSource: RelationSource = {
  useOptions: useGroupOptions,
  useSelected: useSelectedGroup,
}
