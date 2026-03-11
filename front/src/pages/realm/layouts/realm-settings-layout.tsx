import { RouterParams } from '@/routes/router'
import { useMemo } from 'react'
import { useLocation, useParams } from 'react-router'
import PageRealmSettings from '../ui/page-realm-settings'
import { mapRealms } from '@/api/core.mapper'
import useRealmStore from '@/store/realm.store'

export default function RealmsSettingsLayout() {
  const { realm_name } = useParams<RouterParams>()

  const { pathname } = useLocation()

  const { userRealms } = useRealmStore()
  const realm = mapRealms(userRealms).find((item) => item.name === realm_name)
  const tab = useMemo(() => {
    const pathParts = pathname.split('/')
    const lastPart = pathParts[pathParts.length - 1]
    const validTabs = ['general', 'login', 'email', 'security', 'webhooks']
    return validTabs.includes(lastPart) ? lastPart : 'general'
  }, [pathname])

  if (!realm) return null

  return <PageRealmSettings realm={realm} tab={tab} />
}
