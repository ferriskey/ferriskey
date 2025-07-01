import { Outlet, useParams } from 'react-router'
import { AppSidebar } from '../app-sidebar'
import { SidebarInset, SidebarProvider } from '../ui/sidebar'
import { useGetUserRealmsQuery } from '@/api/realm.api'
import { RouterParams } from '@/routes/router'
import useRealmStore from '@/store/realm.store'
import { useEffect } from 'react'
import GithubStarModal from '../github-star-modal'

export default function Layout() {
  const { realm_name } = useParams<RouterParams>()
  const { setUserRealms } = useRealmStore()
  const { data } = useGetUserRealmsQuery({ realm: realm_name ?? 'master' })

  useEffect(() => {
    if (data) {
      setUserRealms(data)
    }
  }, [data, setUserRealms])

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <Outlet />
      </SidebarInset>

      <GithubStarModal />
    </SidebarProvider>
  )
}
