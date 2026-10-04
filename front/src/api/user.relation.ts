import { useParams } from 'react-router'
import type { RelationOption, RelationSource } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import type { Schemas } from './api.client'
import { USER_SEARCH_LIMIT, useGetUser, useGetUsers } from './user.api'

const toOption = (user: Schemas.User): RelationOption => ({
  id: user.id,
  label: user.username,
  sublabel: user.email ?? undefined,
})

function useRealm() {
  const { realm_name } = useParams<RouterParams>()
  return realm_name ?? 'master'
}

function useUserOptions(search: string) {
  const realm = useRealm()
  const { data, isLoading } = useGetUsers({
    realm,
    query: { username: search.trim() || undefined, limit: USER_SEARCH_LIMIT },
  })
  return { options: (data?.data ?? []).map(toOption), loading: isLoading }
}

function useSelectedUser(id: string | undefined) {
  const realm = useRealm()
  const { data } = useGetUser({ realm, userId: id })
  return data?.data ? toOption(data.data) : undefined
}

export const userRelationSource: RelationSource = {
  useOptions: useUserOptions,
  useSelected: useSelectedUser,
}
